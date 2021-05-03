use anyhow::Context;
use anyhow::Result;
use cargo::core::GitReference;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, ProxyOptions, RemoteCallbacks, Repository, RepositoryInitOptions};
use remove_dir_all::remove_dir_all;
use std::borrow::Cow;
use std::env::current_dir;
use std::path::{Path, PathBuf};

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
}

impl<'a> GitConfig<'a> {
    /// Creates a new `GitConfig` by parsing `git` as a URL or a local path.
    pub fn new(git: Cow<'a, str>, branch: Option<String>) -> Result<Self> {
        let (remote, kind) = match determine_repo_kind(git.as_ref()) {
            RepoKind::Invalid => anyhow::bail!("Invalid git remote '{}'", &git),
            RepoKind::LocalFolder => {
                let given_path = Path::new(git.as_ref());
                let mut git_path = PathBuf::new();
                if given_path.is_relative() {
                    git_path.push(current_dir()?);
                    git_path.push(given_path);
                    if !git_path.exists() {
                        anyhow::bail!(
                            "Failed parsing git remote {:?}: path {:?} doesn't exist",
                            &git,
                            &git_path
                        );
                    }
                } else {
                    git_path.push(git.as_ref());
                }
                (
                    format!("file://{}", git_path.display()).into(),
                    RepoKind::LocalFolder,
                )
            }
            k => (git, k),
        };

        Ok(GitConfig {
            remote,
            kind,
            branch: branch
                .map(GitReference::Branch)
                .unwrap_or(GitReference::DefaultBranch),
        })
    }

    /// Creates a new `GitConfig`, first with `new` and then as a GitHub `owner/repo` remote, like
    /// [hub].
    ///
    /// [hub]: https://github.com/github/hub
    pub fn new_abbr(git: Cow<'a, str>, branch: Option<String>) -> Result<Self> {
        Self::new(git.clone(), branch.clone()).or_else(|_| {
            let full_remote = format!("https://github.com/{}.git", &git);
            Self::new(full_remote.into(), branch)
        })
    }
}

fn git_ssh_credentials_callback<'a>() -> Result<RemoteCallbacks<'a>> {
    let mut priv_key = dirs::home_dir().context("$HOME was not set")?;
    priv_key.push(".ssh/id_rsa");
    let mut cb = RemoteCallbacks::new();
    cb.credentials(
        move |_url, username_from_url: Option<&str>, _allowed_types| {
            Cred::ssh_key(username_from_url.unwrap_or("git"), None, &priv_key, None)
        },
    );
    Ok(cb)
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

pub(crate) fn create(project_dir: &Path, args: GitConfig) -> Result<String> {
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
            let callbacks = git_ssh_credentials_callback()?;
            fo.remote_callbacks(callbacks);
        }
        RepoKind::Invalid => {
            unreachable!()
        }
    }
    builder.fetch_options(fo);

    let repo = builder.clone(args.remote.as_ref(), project_dir)?;
    let branch = get_branch_name_repo(&repo)?;
    init_all_submodules(&repo)?;
    remove_history(project_dir)?;

    Ok(branch)
}

fn remove_history(project_dir: &Path) -> Result<()> {
    remove_dir_all(project_dir.join(".git")).context("Error cleaning up cloned template")?;
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
    if remote_url.starts_with("ssh://") || remote_url.starts_with("git@") {
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
            (
                "ssh://git@github.com:cargo-generate/cargo-generate.git",
                RepoKind::RemoteSsh,
            ),
            ("./", RepoKind::LocalFolder),
            ("ftp://foobar.bak", RepoKind::Invalid),
        ] {
            let kind = determine_repo_kind(u);
            assert_eq!(&kind, k, "{} is not a {:?}", u, k);
        }
    }

    #[test]
    fn should_not_fail_for_ssh_remote_urls() {
        let config = GitConfig::new(REPO_URL_SSH.into(), None).unwrap();
        assert_eq!(config.kind, RepoKind::RemoteSsh);
    }

    #[test]
    #[should_panic(expected = "Invalid git remote 'aslkdgjlaskjdglskj'")]
    fn should_fail_for_non_existing_local_path() {
        GitConfig::new("aslkdgjlaskjdglskj".into(), None).unwrap();
    }

    #[test]
    fn should_support_a_local_relative_path() {
        let remote: String = GitConfig::new("src".into(), None).unwrap().remote.into();
        assert!(
            remote.ends_with("/src"),
            "remote {} ends with /src",
            &remote
        );

        #[cfg(unix)]
        assert!(
            remote.starts_with("file:///"),
            "remote {} starts with file:///",
            &remote
        );
    }

    #[test]
    #[cfg(unix)]
    fn should_support_a_local_absolute_path() {
        // Absolute path.
        // If this fails because you cloned this repository into a non-UTF-8 directory... all
        // I can say is you probably had it comin'.
        let remote: String =
            GitConfig::new(current_dir().unwrap().display().to_string().into(), None)
                .unwrap()
                .remote
                .into();
        assert!(
            remote.starts_with("file:///"),
            "remote {} starts with file:///",
            remote
        );
    }

    #[test]
    fn should_test_happy_path() {
        // Remote HTTPS URL.
        let cfg = GitConfig::new(REPO_URL.into(), Some("main".to_owned())).unwrap();

        assert_eq!(cfg.remote.as_ref(), Url::parse(REPO_URL).unwrap().as_str());
        assert_eq!(cfg.branch, GitReference::Branch("main".to_owned()));
    }

    #[test]
    fn should_support_abbreviated_repository_short_urls_like() {
        assert_eq!(
            GitConfig::new_abbr("cargo-generate/cargo-generate".into(), None)
                .unwrap()
                .remote
                .as_ref(),
            Url::parse(REPO_URL).unwrap().as_str()
        );
    }
}
