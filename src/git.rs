use anyhow::{Context, Result};
use cargo::core::GitReference;
use cargo::sources::git::GitRemote;
use cargo::util::config::Config;
use git2::{Repository as GitRepository, RepositoryInitOptions};
use remove_dir_all::remove_dir_all;
use std::env::current_dir;
use std::path::Path;
use std::path::PathBuf;
use tempfile::Builder;
use url::{ParseError, Url};

pub(crate) struct GitConfig {
    remote: Url,
    branch: GitReference,
}

impl GitConfig {
    /// Creates a new `GitConfig` by parsing `git` as a URL or a local path.
    pub fn new(git: &str, branch: Option<String>) -> Result<Self> {
        let remote = match Url::parse(git) {
            Ok(u) => u,
            Err(ParseError::RelativeUrlWithoutBase) => {
                let given_path = Path::new(git);
                let mut git_path = PathBuf::new();
                if given_path.is_relative() {
                    git_path.push(current_dir()?);
                    git_path.push(given_path);
                    if !git_path.exists() {
                        anyhow::bail!(
                            "Failed parsing git remote {:?}: path {:?} doesn't exist",
                            git,
                            &git_path
                        );
                    }
                } else {
                    git_path.push(git)
                }
                Url::from_file_path(&git_path).map_err(|()| {
                    anyhow::format_err!(
                        "Failed parsing git remote (also tried as a file path): {}",
                        &git
                    )
                })?
            }
            Err(err) => anyhow::bail!("Failed parsing git remote {:?}: {}", git, err),
        };

        Ok(GitConfig {
            remote,
            branch: branch
                .map(GitReference::Branch)
                .unwrap_or(GitReference::DefaultBranch),
        })
    }

    /// Creates a new `GitConfig`, first with `new` and then as a GitHub `owner/repo` remote, like
    /// [hub].
    ///
    /// [hub]: https://github.com/github/hub
    pub fn new_abbr(git: &str, branch: Option<String>) -> Result<Self> {
        Self::new(git, branch.clone()).or_else(|e| {
            Self::new(&format!("https://github.com/{}.git", git), branch).map_err(|_| e)
        })
    }
}

pub(crate) fn create(project_dir: &Path, args: GitConfig) -> Result<String> {
    let temp = Builder::new().prefix(project_dir).tempdir()?;
    let config = Config::default()?;
    let remote = GitRemote::new(&args.remote);

    let ((db, rev), branch_name) = match &args.branch {
        GitReference::Branch(branch_name) => (
            remote.checkout(&temp.path(), None, &args.branch, None, &config)?,
            branch_name.clone(),
        ),
        GitReference::DefaultBranch => {
            // Cargo has a specific behavior for now for handling the "default" branch. It forces
            // it to the branch named "master" even if the actual default branch of the repository
            // is something else. They intent to change this behavior in the future but they don't
            // want to break the compatibility.
            //
            // See issues:
            //  - https://github.com/rust-lang/cargo/issues/8364
            //  - https://github.com/rust-lang/cargo/issues/8468
            let repo = git2::Repository::init(&temp.path())?;
            let mut git_remote = repo.remote_anonymous(remote.url().as_str())?;
            git_remote.connect(git2::Direction::Fetch)?;
            let default_branch = git_remote.default_branch()?;
            let branch_name = default_branch
                .as_str()
                .unwrap_or("refs/heads/master")
                .replace("refs/heads/", "");
            (
                remote.checkout(
                    &temp.path(),
                    None,
                    &GitReference::Branch(branch_name.clone()),
                    None,
                    &config,
                )?,
                branch_name,
            )
        }
        _ => unreachable!(),
    };

    // This clones the remote and handles all the submodules
    db.copy_to(rev, project_dir, &config)?;
    Ok(branch_name)
}

pub(crate) fn remove_history(project_dir: &Path) -> Result<()> {
    remove_dir_all(project_dir.join(".git")).context("Error cleaning up cloned template")?;
    Ok(())
}

pub fn init(project_dir: &Path, branch: &str) -> Result<GitRepository> {
    let mut opts = RepositoryInitOptions::new();
    opts.bare(false);
    opts.initial_head(branch);
    GitRepository::init_opts(project_dir, &opts).context("Couldn't init new repository")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gitconfig_new_test() {
        // Remote HTTPS URL.
        let cfg = GitConfig::new(
            "https://github.com/ashleygwilliams/cargo-generate.git",
            Some("main".to_owned()),
        )
        .unwrap();

        assert_eq!(
            cfg.remote,
            Url::parse("https://github.com/ashleygwilliams/cargo-generate.git").unwrap()
        );
        assert_eq!(cfg.branch, GitReference::Branch("main".to_owned()));

        // Fails because "ashleygwilliams" is a "bad port number". Out of scope for now -- not sure
        // how common SSH URLs are at this point anyways...?
        assert!(GitConfig::new(
            "ssh://git@github.com:ashleygwilliams/cargo-generate.git",
            None,
        )
        .is_err());

        // Local path doesn't exist.
        assert!(GitConfig::new("aslkdgjlaskjdglskj", None).is_err());

        // Local path does exist.
        let remote = GitConfig::new("src", None)
            .unwrap()
            .remote
            .into_string();
        assert!(
            remote.ends_with("/src"),
            format!("remote {} ends with /src", &remote)
        );

        #[cfg(unix)]
        {
            assert!(
                remote.starts_with("file:///"),
                format!("remote {} starts with file:///", &remote)
            );
        }

        #[cfg(unix)]
        {
            // Absolute path.
            // If this fails because you cloned this repository into a non-UTF-8 directory... all
            // I can say is you probably had it comin'.
            let remote = GitConfig::new(current_dir().unwrap().to_str().unwrap(), None)
                .unwrap()
                .remote
                .into_string();
            assert!(
                remote.starts_with("file:///"),
                format!("remote {} starts with file:///", &remote)
            );
        }
    }

    #[test]
    fn gitconfig_new_abbr_test() {
        // Abbreviated owner/repo form
        assert_eq!(
            GitConfig::new_abbr("ashleygwilliams/cargo-generate", None)
                .unwrap()
                .remote,
            Url::parse("https://github.com/ashleygwilliams/cargo-generate.git").unwrap()
        );
    }
}
