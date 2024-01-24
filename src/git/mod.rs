//! Handle `--git` and related flags

use std::path::Path;
use std::{io, ops::Sub, thread::sleep, time::Duration};

use anyhow::Result;
use auth_git2::GitAuthenticator;
use console::style;
use git2::{build::RepoBuilder, FetchOptions, ProxyOptions, Repository, RepositoryInitOptions};
use log::warn;
use remove_dir_all::remove_dir_all;
pub use utils::clone_git_template_into_temp;

mod gitconfig;
mod utils;

pub use utils::{tmp_dir, try_get_branch_from_path};

// cargo-generate (as application) want from git module:
// 1. cloning remote
// 2. initialize freshly generated template
// 3. remove history from cloned template

// Assumptions:
// * `--git <url>` should only be parse in the same way as `git clone <url>` would
// * submodules can be cloned by setting the submodules field to true.
// * `.git` should be removed to make clear repository
// * if `<url>` is the local path on system the clone should also be done the same way as `git clone` there is `--path`
//    for different behavior

// basically we want to call:
// git clone --recurse-submodules --depth 1 --branch <branch> <url> <tmp_dir>
// with --recurse-submodules being optional.

type Git2Result<T> = Result<T, git2::Error>;

struct RepoCloneBuilder<'cb> {
    builder: RepoBuilder<'cb>,
    authenticator: GitAuthenticator,
    url: String,
    skip_submodules: bool,
}

impl<'cb> RepoCloneBuilder<'cb> {
    pub fn new(url: &str, skip_submodules: bool) -> Result<Self> {
        let url = gitconfig::find_gitconfig()?.map_or_else(
            || url.to_owned(),
            |gitcfg| {
                gitconfig::resolve_instead_url(url, gitcfg)
                    .expect("correct configuration")
                    .unwrap_or_else(|| url.to_owned())
            },
        );

        Ok(Self {
            builder: RepoBuilder::new(),
            authenticator: GitAuthenticator::default(),
            url,
            skip_submodules,
        })
    }

    pub fn new_with(
        url: &str,
        branch: Option<&str>,
        identity_path: Option<&Path>,
        submodules: bool,
    ) -> Result<Self> {
        let mut builder = Self::new(url, submodules)?;
        if let Some(branch) = branch {
            builder.set_branch(branch);
        }

        if let Some(identity_path) = identity_path {
            builder.set_identity(identity_path)?;
        }

        Ok(builder)
    }

    pub fn set_identity(&mut self, identity_path: &Path) -> Result<()> {
        let identity_path = utils::canonicalize_path(identity_path)?;
        log::info!(
            "{} `{}` {}",
            style("Using private key:").bold(),
            style(format_args!("{}", identity_path.display()))
                .bold()
                .yellow(),
            style("for git-ssh checkout").bold()
        );
        self.authenticator = GitAuthenticator::new_empty()
            .add_ssh_key_from_file(identity_path, None)
            .prompt_ssh_key_password(true);
        Ok(())
    }

    pub fn set_branch(&mut self, branch: &str) {
        self.builder.branch(branch);
    }

    fn clone(self, dest_path: &Path) -> Result<Repository> {
        let config = git2::Config::open_default()?;

        let mut proxy_options = ProxyOptions::new();
        proxy_options.auto();

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(self.authenticator.credentials(&config));

        let mut fetch_options = FetchOptions::new();
        fetch_options.proxy_options(proxy_options);
        fetch_options.remote_callbacks(callbacks);

        let mut builder = self.builder;
        builder.fetch_options(fetch_options);

        builder
            .clone(&self.url, dest_path)
            .map_err(anyhow::Error::from)
    }

    pub fn clone_with_submodules(self, dest_path: &Path) -> Result<Repository> {
        let authenticator = Clone::clone(&self.authenticator);
        let skip_submodules = self.skip_submodules;
        let repo = self.clone(dest_path)?;
        if skip_submodules {
            return Ok(repo);
        }

        let config = repo.config()?;

        for mut sub in repo.submodules()? {
            let mut proxy_options = ProxyOptions::new();
            proxy_options.auto();

            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(authenticator.credentials(&config));

            let mut fetch_options = FetchOptions::new();
            fetch_options.proxy_options(proxy_options);
            fetch_options.remote_callbacks(callbacks);

            let mut update_options = git2::SubmoduleUpdateOptions::new();
            update_options.fetch(fetch_options);
            sub.update(true, Some(&mut update_options))?;
        }

        Ok(repo)
    }
}

/// Init project_dir with fresh repository on branch
///
/// Arguments:
/// - `force` - enforce a fresh git init
pub fn init(project_dir: &Path, branch: Option<&str>, force: bool) -> Git2Result<Repository> {
    fn just_init(project_dir: &Path, branch: Option<&str>) -> Git2Result<Repository> {
        let mut opts = RepositoryInitOptions::new();
        opts.bare(false);
        if let Some(branch) = branch {
            opts.initial_head(branch);
        }
        Repository::init_opts(project_dir, &opts)
    }

    Repository::discover(project_dir).map_or_else(
        |_| just_init(project_dir, branch),
        |repo| {
            if force {
                Repository::open(project_dir).or_else(|_| just_init(project_dir, branch))
            } else {
                Ok(repo)
            }
        },
    )
}

/// remove context of repository by removing `.git` from filesystem
pub fn remove_history(project_dir: &Path) -> io::Result<()> {
    let git_dir = project_dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        let mut attempt = 0_u8;

        loop {
            attempt += 1;
            if let Err(e) = remove_dir_all(&git_dir) {
                if attempt == 5 {
                    return Err(e);
                }

                if e.to_string().contains("The process cannot access the file because it is being used by another process.") {
                    let wait_for = Duration::from_secs(2_u64.pow(attempt.sub(1).into()));
                    warn!("Git history cleanup failed with a windows process blocking error. [Retry in {:?}]", wait_for);
                    sleep(wait_for);
                } else {
                    return Err(e);
                }
            } else {
                return Ok(());
            }
        }
    } else {
        //FIXME should we assume this is expected by caller?
        // panic!("tmp panic");
        Ok(())
    }
}
