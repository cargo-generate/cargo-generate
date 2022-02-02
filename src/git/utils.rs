use crate::git::remote::{GitRemote, GitUrlAndConfig};
use crate::warn;
use crate::{copy_dir_all, info};

use crate::git::identity_path::IdentityPath;
use anyhow::Context;
use anyhow::Result;
use console::style;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, ProxyOptions, RemoteCallbacks, Repository, RepositoryInitOptions};
use git2::{ErrorClass, ErrorCode};
use git_config::file::GitConfig as GitConfigParser;
use git_config::parser::Key;
use remove_dir_all::remove_dir_all;
use std::borrow::Cow;
use std::ops::{Add, Sub};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RepoKind {
    LocalFolder,
    RemoteHttp,
    RemoteHttps,
    RemoteSsh,
    RemoteGit,
    Invalid,
}

/// Information to find a specific commit in a Git repository.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GitReference {
    /// From a branch.
    Branch(String),
    /// The default branch of the repository, the reference named `HEAD`.
    DefaultBranch,
}

pub struct GitConfig<'a> {
    remote: GitRemote<'a>,
    branch: GitReference,
    identity: Option<PathBuf>,
}

impl<'a> GitConfig<'a> {
    /// Creates a new `GitConfig` by parsing `git` as a URL or a local path.
    pub fn new(
        git: impl AsRef<str> + 'a,
        branch: Option<String>,
        identity: Option<PathBuf>,
    ) -> Result<Self> {
        let git = Cow::from(git.as_ref().to_string());
        let gitconfig = find_gitconfig()?;
        let remote = if let Some(gitconfig) = gitconfig {
            GitRemote::try_from(GitUrlAndConfig(git, gitconfig))?
        } else {
            GitRemote::try_from(git)?
        };

        Ok(GitConfig {
            remote,
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
        git: impl AsRef<str>,
        branch: Option<String>,
        identity: Option<PathBuf>,
    ) -> Result<Self> {
        let full_remote = abbreviated_git_url_to_full_remote(&git);
        Self::new(full_remote, branch.clone(), identity.clone()).or_else(|_| {
            let full_remote = abbreviated_git_url_to_full_remote(format!("gh:{}", git.as_ref()));
            Self::new(full_remote, branch, identity)
        })
    }
}

pub fn abbreviated_git_url_to_full_remote(git: impl AsRef<str>) -> String {
    let git = git.as_ref();
    if git.len() >= 3 {
        match &git[..3] {
            "gl:" => format!("https://gitlab.com/{}.git", &git[3..]),
            "bb:" => format!("https://bitbucket.org/{}.git", &git[3..]),
            "gh:" => format!("https://github.com/{}.git", &git[3..]),
            _ => git.to_owned(),
        }
    } else {
        git.to_owned()
    }
}

pub fn create(project_dir: &Path, args: GitConfig) -> Result<String> {
    let branch = git_clone_all(project_dir, args)?;
    remove_history(project_dir, None)?;

    Ok(branch)
}

/// deals with `~/` and `$HOME/` prefixes
pub fn canonicalize_path(p: impl AsRef<Path>) -> Result<PathBuf> {
    let p = p.as_ref();
    let p = if p.starts_with("~/") {
        home()?.join(p.strip_prefix("~/")?)
    } else if p.starts_with("$HOME/") {
        home()?.join(p.strip_prefix("$HOME/")?)
    } else {
        p.to_path_buf()
    };

    p.canonicalize().context("path does not exist")
}

#[test]
fn should_canonicalize() {
    #[cfg(target_os = "macos")]
    {
        assert!(canonicalize_path(&PathBuf::from("../"))
            .unwrap()
            .starts_with("/Users/"));

        assert!(canonicalize_path(&PathBuf::from("$HOME/"))
            .unwrap()
            .starts_with("/Users/"));
    }
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
fn get_private_key_path(identity: Option<PathBuf>) -> Result<IdentityPath> {
    let private_key = identity.unwrap_or(home()?.join(".ssh/id_rsa"));
    private_key.try_into()
}

fn git_ssh_credentials_callback<'a>(identity: Option<PathBuf>) -> Result<RemoteCallbacks<'a>> {
    let private_key = get_private_key_path(identity)?;
    info!(
        "{} `{}` {}",
        style("Using private key:").bold(),
        style(format!("{}", &private_key)).bold().yellow(),
        style("for git-ssh checkout").bold()
    );
    let mut cb = RemoteCallbacks::new();
    cb.credentials(
        move |_url, username_from_url: Option<&str>, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                private_key.as_ref(),
                None,
            )
        },
    );
    Ok(cb)
}

/// home path wrapper
pub fn home() -> Result<PathBuf> {
    canonicalize_path(&dirs::home_dir().context("$HOME was not set")?)
}

/// thanks to @extrawurst for pointing this out
/// <https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/branch/mod.rs#L38>
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
    match args.remote.as_ref() {
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
        RepoKind::RemoteGit => {
            // todo: verify if that would just work as is
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
            {
                // super wired that the windows libgit2 does behave different for this case,
                // both things actually mean the same!
                #[cfg(windows)]
                if e.class() == ErrorClass::Http
                    && e.code() == ErrorCode::GenericError
                    && e.message().contains("request failed with status code: 401")
                {
                    return Err(anyhow::Error::msg(
                        "Please check if the Git user / repository exists.",
                    ));
                }
                #[cfg(not(windows))]
                if e.code() == ErrorCode::Auth && e.class() == ErrorClass::Http {
                    return Err(anyhow::Error::msg(
                        "Please check if the Git user / repository exists.",
                    ));
                }
            }
            if e.code() != ErrorCode::NotFound {
                return Err(e.into());
            }

            let path: &str = args.remote.as_ref();
            let path = Path::new(path);
            if !path.exists() || !path.is_dir() {
                return Err(e.into());
            }

            warn!("Template does not seem to be a git repository, using as a plain folder");
            copy_dir_all(path, project_dir)?;
            Ok("".to_string())
        }
    }
}

pub fn remove_history(project_dir: &Path, attempt: Option<u8>) -> Result<()> {
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
                let wait_for = Duration::from_secs(2_u64.pow(attempt.sub(1).into()));
                warn!("Git history cleanup failed with a windows process blocking error. [Retry in {:?}]", wait_for);
                sleep(wait_for);
                remove_history(project_dir, Some(attempt.add(1)))?
            }
        }
    }
    Ok(())
}

pub fn init(project_dir: &Path, branch: &str, force: bool) -> Result<Repository> {
    fn just_init(project_dir: &Path, branch: &str) -> Result<Repository> {
        let mut opts = RepositoryInitOptions::new();
        opts.bare(false);
        opts.initial_head(branch);
        Repository::init_opts(project_dir, &opts).context("Couldn't init new repository")
    }

    match Repository::discover(project_dir) {
        Ok(repo) => {
            if force {
                Repository::open(project_dir).or_else(|_| just_init(project_dir, branch))
            } else {
                Ok(repo)
            }
        }
        Err(_) => just_init(project_dir, branch),
    }
}

/// determines what kind of repository we got
pub fn determine_repo_kind(remote_url: &str) -> RepoKind {
    if remote_url.starts_with("git@") || remote_url.starts_with("ssh://") {
        RepoKind::RemoteSsh
    } else if remote_url.starts_with("git://") {
        RepoKind::RemoteGit
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

pub fn find_gitconfig() -> Result<Option<PathBuf>> {
    let gitconfig = home().map(|home| home.join(".gitconfig"))?;
    if gitconfig.exists() {
        return Ok(Some(gitconfig));
    }

    Ok(None)
}

/// trades urls, to replace a given repo remote url with the right on based
/// on the `[url]` section in the `~/.gitconfig`
pub fn resolve_instead_url(
    remote: impl AsRef<str>,
    gitconfig: impl AsRef<Path>,
) -> Result<Option<String>> {
    let gitconfig = gitconfig.as_ref();
    let remote = remote.as_ref().to_string();
    let config = GitConfigParser::open(gitconfig).context("Cannot read or parse .gitconfig")?;
    Ok(config
        .sections_by_name_with_header("url")
        .iter()
        .map(|(head, body)| {
            let url = head.subsection_name.as_ref();
            let instead_of = body
                .value(&Key::from("insteadOf"))
                .map(|x| std::str::from_utf8(&x[..]).unwrap().to_owned());
            (instead_of, url)
        })
        .filter(|(old, new)| new.is_some() && old.is_some())
        .map(|(old, new)| {
            let old = old.unwrap();
            let new = new.unwrap().to_string();
            remote
                .starts_with(old.as_str())
                .then(|| remote.replace(old.as_str(), new.as_str()))
        })
        .flatten()
        .next())
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
        let config = GitConfig::new(REPO_URL_SSH, None, None).unwrap();
        let kind: &RepoKind = config.remote.as_ref();
        assert_eq!(kind, &RepoKind::RemoteSsh);
    }

    #[test]
    #[should_panic(expected = "Invalid git remote 'aslkdgjlaskjdglskj'")]
    fn should_fail_for_non_existing_local_path() {
        GitConfig::new("aslkdgjlaskjdglskj", None, None).unwrap();
    }

    #[test]
    fn should_support_a_local_relative_path() {
        let config = GitConfig::new("src", None, None).unwrap();
        let remote: &str = config.remote.as_ref();
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
        let config =
            GitConfig::new(current_dir().unwrap().display().to_string(), None, None).unwrap();
        let remote: &str = config.remote.as_ref();
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
        let cfg = GitConfig::new(REPO_URL, Some("main".to_owned()), None).unwrap();
        let url: &str = cfg.remote.as_ref();

        assert_eq!(url, Url::parse(REPO_URL).unwrap().as_str());
        assert_eq!(cfg.branch, GitReference::Branch("main".to_owned()));
    }

    #[test]
    fn should_support_abbreviated_repository_short_urls_like() {
        let config = GitConfig::new_abbr("cargo-generate/cargo-generate", None, None).unwrap();
        let url: &str = config.remote.as_ref();
        assert_eq!(url, Url::parse(REPO_URL).unwrap().as_str());
    }

    #[test]
    fn should_support_abbreviated_repository_short_urls_like_for_github() {
        let config = GitConfig::new_abbr("gh:cargo-generate/cargo-generate", None, None).unwrap();
        let url: &str = config.remote.as_ref();
        assert_eq!(url, Url::parse(REPO_URL).unwrap().as_str());
    }

    #[test]
    fn should_support_bb_gl_gh_abbreviations() {
        assert_eq!(
            &abbreviated_git_url_to_full_remote("gh:foo/bar"),
            "https://github.com/foo/bar.git"
        );
        assert_eq!(
            &abbreviated_git_url_to_full_remote("bb:foo/bar"),
            "https://bitbucket.org/foo/bar.git"
        );
        assert_eq!(
            &abbreviated_git_url_to_full_remote("gl:foo/bar"),
            "https://gitlab.com/foo/bar.git"
        );
        assert_eq!(&abbreviated_git_url_to_full_remote("foo/bar"), "foo/bar");
    }

    #[test]
    fn should_resolve_instead_url() {
        let sample_config = r#"
[url "ssh://git@github.com:"]
    insteadOf = https://github.com/
"#;
        let where_gitconfig_lives = tempfile::tempdir().unwrap();
        let gitconfig = where_gitconfig_lives.path().join(".gitconfig");
        std::fs::write(&gitconfig, sample_config).unwrap();

        // SSH, aka git@github.com: or ssh://git@github.com/
        let x = resolve_instead_url("https://github.com/foo/bar.git", &gitconfig).unwrap();
        assert_eq!(x.unwrap().as_str(), "ssh://git@github.com:foo/bar.git")
    }
}
