use crate::copy_dir_all;
use crate::emoji;
use crate::warn;
use anyhow::Context;
use anyhow::Result;
use cargo::core::GitReference;
use console::style;
use git2::build::RepoBuilder;
use git2::ErrorCode;
use git2::{Cred, FetchOptions, ProxyOptions, RemoteCallbacks, Repository, RepositoryInitOptions};
use remove_dir_all::remove_dir_all;
use std::borrow::Cow;
use std::ops::{Add, Deref, Sub};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum RepoKind {
    LocalFolder,
    RemoteHttp,
    RemoteHttps,
    RemoteSsh,
    Invalid,
}

pub(crate) struct GitConfig<'a> {
    remote: Cow<'a, str>,
    branch: GitReference,
    kind: RepoKind,
    identity: Option<PathBuf>,
}

impl<'a> GitConfig<'a> {
    /// Creates a new `GitConfig` by parsing `git` as a URL or a local path.
    pub fn new(
        git: Cow<'a, str>,
        branch: Option<String>,
        identity: Option<PathBuf>,
    ) -> Result<Self> {
        let (remote, kind) = match determine_repo_kind(git.as_ref()) {
            RepoKind::Invalid => anyhow::bail!("Invalid git remote '{}'", &git),
            RepoKind::LocalFolder => {
                let full_path = canonicalize_path(git.deref().as_ref())?;
                if !full_path.exists() {
                    anyhow::bail!("The given git remote {:?} does not exist.", &git,);
                }
                (
                    full_path.display().to_string().into(),
                    RepoKind::LocalFolder,
                )
            }
            k => (git, k),
        };

        Ok(GitConfig {
            remote,
            kind,
            identity,
            branch: branch
                .map(GitReference::Branch)
                .unwrap_or(GitReference::DefaultBranch),
        })
    }

    /// Creates a new `GitConfig`, first with `new` and then as a GitHub `owner/repo` remote, like
    /// [hub].
    ///
    /// [hub]: https://github.com/github/hub
    pub fn new_abbr(
        git: Cow<'a, str>,
        branch: Option<String>,
        identity: Option<PathBuf>,
    ) -> Result<Self> {
        Self::new(git.clone(), branch.clone(), identity.clone()).or_else(|_| {
            let full_remote = format!("https://github.com/{}.git", &git);
            Self::new(full_remote.into(), branch, identity)
        })
    }
}

pub(crate) fn create(project_dir: &Path, args: GitConfig) -> Result<String> {
    let branch = git_clone_all(project_dir, args)?;
    remove_history(project_dir, None)?;

    Ok(branch)
}

fn canonicalize_path(p: &Path) -> Result<PathBuf> {
    let p = if p.to_str().unwrap().starts_with("~/") {
        home()?.join(p.strip_prefix("~/").unwrap())
    } else {
        p.to_path_buf()
    };

    p.canonicalize().context("path does not exist")
}

#[test]
fn should_canonicalize() {
    #[cfg(target_os = "macos")]
    assert!(canonicalize_path(&PathBuf::from("../"))
        .unwrap()
        .starts_with("/Users/"));
    #[cfg(target_os = "linux")]
    assert_eq!(
        canonicalize_path(&PathBuf::from("../")).ok(),
        std::env::current_dir()
            .unwrap()
            .parent()
            .map(|p| p.to_path_buf())
    );
    #[cfg(windows)]
    assert!(canonicalize_path(&PathBuf::from("../"))
        .unwrap()
        // not a bug, a feature:
        // https://stackoverflow.com/questions/41233684/why-does-my-canonicalized-path-get-prefixed-with
        .to_str()
        .unwrap()
        .starts_with("\\\\?\\"));
}

/// takes care of `~/` paths, defaults to `$HOME/.ssh/id_rsa` and resolves symlinks.
fn get_private_key_path(identity: Option<PathBuf>) -> Result<PathBuf> {
    let private_key = identity.unwrap_or(home()?.join(".ssh/id_rsa"));

    canonicalize_path(&private_key).context("private key path was incorrect")
}

fn git_ssh_credentials_callback<'a>(identity: Option<PathBuf>) -> Result<RemoteCallbacks<'a>> {
    let private_key = get_private_key_path(identity)?;
    println!(
        "{} {} `{}` {}",
        emoji::INFO,
        style("Using private key:").bold(),
        style(pretty_path(&private_key)?).bold().yellow(),
        style("for git-ssh checkout").bold()
    );
    let mut cb = RemoteCallbacks::new();
    cb.credentials(
        move |_url, username_from_url: Option<&str>, _allowed_types| {
            Cred::ssh_key(username_from_url.unwrap_or("git"), None, &private_key, None)
        },
    );
    Ok(cb)
}

/// home path wrapper
fn home() -> Result<PathBuf> {
    canonicalize_path(&dirs::home_dir().context("$HOME was not set")?)
}

#[test]
fn should_pretty_path() {
    let p = pretty_path(home().unwrap().as_path().join(".cargo").as_path()).unwrap();
    #[cfg(unix)]
    assert_eq!(p, "$HOME/.cargo");
    #[cfg(windows)]
    assert_eq!(p, "%userprofile%\\.cargo");
}

/// prevents from long stupid paths, and replace the home path by the literal `$HOME`
fn pretty_path(a: &Path) -> Result<String> {
    #[cfg(unix)]
    let home_var = "$HOME";
    #[cfg(windows)]
    let home_var = "%userprofile%";
    Ok(a.display()
        .to_string()
        .replace(&home()?.display().to_string(), home_var))
}

/// thanks to @extrawurst for pointing this out
/// https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/branch/mod.rs#L38
fn get_branch_name_repo(repo: &Repository) -> Result<String> {
    let iter = repo.branches(None)?;

    for b in iter {
        let b = b?;

        if b.0.is_head() {
            let name = b.0.name()?.unwrap_or("");
            return Ok(name.into());
        }
    }

    anyhow::bail!("A repo has no Head")
}

fn init_all_submodules(repo: &Repository) -> Result<()> {
    for mut sub in repo.submodules().unwrap() {
        sub.update(true, None)?;
    }

    Ok(())
}

fn git_clone_all(project_dir: &Path, args: GitConfig) -> Result<String> {
    let mut builder = RepoBuilder::new();
    if let GitReference::Branch(branch_name) = &args.branch {
        builder.branch(branch_name.as_str());
    }

    let mut fo = FetchOptions::new();
    match args.kind {
        RepoKind::LocalFolder => {}
        RepoKind::RemoteHttp | RepoKind::RemoteHttps => {
            let mut proxy = ProxyOptions::new();
            proxy.auto();
            fo.proxy_options(proxy);
        }
        RepoKind::RemoteSsh => {
            let callbacks = git_ssh_credentials_callback(args.identity)?;
            fo.remote_callbacks(callbacks);
        }
        RepoKind::Invalid => {
            unreachable!()
        }
    }
    builder.fetch_options(fo);

    match builder.clone(args.remote.as_ref(), project_dir) {
        Ok(repo) => {
            let branch = get_branch_name_repo(&repo)?;
            init_all_submodules(&repo)?;
            Ok(branch)
        }
        Err(e) => {
            if e.code() != ErrorCode::NotFound {
                return Err(e.into());
            }

            let path = Path::new(&*args.remote);
            if !path.exists() || !path.is_dir() {
                return Err(e.into());
            }

            warn!("Template does not seem to be a git repository, using as a plain folder");
            copy_dir_all(path, project_dir)?;
            Ok("".to_string())
        }
    }
}

fn remove_history(project_dir: &Path, attempt: Option<u8>) -> Result<()> {
    let git_dir = project_dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        if let Err(e) = remove_dir_all(git_dir) {
            // see https://github.com/cargo-generate/cargo-generate/issues/375
            if e.to_string().contains(
                "The process cannot access the file because it is being used by another process.",
            ) {
                let attempt = attempt.unwrap_or(1);
                if attempt == 5 {
                    warn!("cargo-generate was not able to delete the git history after {} retries. Please delete the `.git` sub-folder manually", attempt);
                    return Ok(());
                }
                let wait_for = Duration::from_secs(2_u64.pow(attempt.sub(1) as u32));
                warn!("Git history cleanup failed with a windows process blocking error. [Retry in {:?}]", wait_for);
                sleep(wait_for);
                remove_history(project_dir, Some(attempt.add(1)))?
            }
        }
    }
    Ok(())
}

pub fn init(project_dir: &Path, branch: &str) -> Result<Repository> {
    Repository::discover(project_dir).or_else(|_| {
        let mut opts = RepositoryInitOptions::new();
        opts.bare(false);
        opts.initial_head(branch);
        Repository::init_opts(project_dir, &opts).context("Couldn't init new repository")
    })
}

/// determines what kind of repository we got
fn determine_repo_kind(remote_url: &str) -> RepoKind {
    if remote_url.starts_with("git@") {
        RepoKind::RemoteSsh
    } else if remote_url.starts_with("http://") {
        RepoKind::RemoteHttp
    } else if remote_url.starts_with("https://") {
        RepoKind::RemoteHttps
    } else if Path::new(remote_url).exists() {
        RepoKind::LocalFolder
    } else {
        RepoKind::Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;
    use url::Url;

    const REPO_URL: &str = "https://github.com/cargo-generate/cargo-generate.git";
    const REPO_URL_SSH: &str = "git@github.com:cargo-generate/cargo-generate.git";

    #[test]
    fn should_determine_repo_kind() {
        for (u, k) in &[
            (REPO_URL, RepoKind::RemoteHttps),
            (
                "http://github.com/cargo-generate/cargo-generate.git",
                RepoKind::RemoteHttp,
            ),
            (REPO_URL_SSH, RepoKind::RemoteSsh),
            ("./", RepoKind::LocalFolder),
            ("ftp://foobar.bak", RepoKind::Invalid),
        ] {
            let kind = determine_repo_kind(u);
            assert_eq!(&kind, k, "{} is not a {:?}", u, k);
        }
    }

    #[test]
    fn should_not_fail_for_ssh_remote_urls() {
        let config = GitConfig::new(REPO_URL_SSH.into(), None, None).unwrap();
        assert_eq!(config.kind, RepoKind::RemoteSsh);
    }

    #[test]
    #[should_panic(expected = "Invalid git remote 'aslkdgjlaskjdglskj'")]
    fn should_fail_for_non_existing_local_path() {
        GitConfig::new("aslkdgjlaskjdglskj".into(), None, None).unwrap();
    }

    #[test]
    fn should_support_a_local_relative_path() {
        let remote: String = GitConfig::new("src".into(), None, None)
            .unwrap()
            .remote
            .into();
        #[cfg(unix)]
        assert!(
            remote.ends_with("/src"),
            "remote {} ends with /src",
            &remote
        );
        #[cfg(windows)]
        assert!(
            remote.ends_with("\\src"),
            "remote {} ends with \\src",
            &remote
        );

        #[cfg(unix)]
        assert!(remote.starts_with('/'), "remote {} starts with /", &remote);
        #[cfg(windows)]
        assert!(
            remote.starts_with("\\\\?\\"),
            "remote {} starts with \\\\?\\",
            &remote
        );
    }

    #[test]
    fn should_support_a_local_absolute_path() {
        // Absolute path.
        // If this fails because you cloned this repository into a non-UTF-8 directory... all
        // I can say is you probably had it comin'.
        let remote: String = GitConfig::new(
            current_dir().unwrap().display().to_string().into(),
            None,
            None,
        )
        .unwrap()
        .remote
        .into();
        #[cfg(unix)]
        assert!(remote.starts_with('/'), "remote {} starts with /", &remote);
        #[cfg(windows)]
        assert!(
            remote.starts_with("\\\\?\\"),
            "remote {} starts with \\\\?\\ then the drive letter",
            &remote
        );
    }

    #[test]
    fn should_test_happy_path() {
        // Remote HTTPS URL.
        let cfg = GitConfig::new(REPO_URL.into(), Some("main".to_owned()), None).unwrap();

        assert_eq!(cfg.remote.as_ref(), Url::parse(REPO_URL).unwrap().as_str());
        assert_eq!(cfg.branch, GitReference::Branch("main".to_owned()));
    }

    #[test]
    fn should_support_abbreviated_repository_short_urls_like() {
        assert_eq!(
            GitConfig::new_abbr("cargo-generate/cargo-generate".into(), None, None)
                .unwrap()
                .remote
                .as_ref(),
            Url::parse(REPO_URL).unwrap().as_str()
        );
    }
}
