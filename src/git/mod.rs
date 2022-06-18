//! Handle `--git` and related flags

use std::path::{Path, PathBuf};
use std::{io, ops::Sub, thread::sleep, time::Duration};

use anyhow::Result;
use git2::{build::RepoBuilder, FetchOptions, ProxyOptions, Repository, RepositoryInitOptions};
use remove_dir_all::remove_dir_all;

pub use utils::clone_git_template_into_temp;

use crate::warn;

mod creds;
mod gitconfig;
mod identity_path;
mod utils;

// cargo-generate (as application) whant from git module:
// 1. cloning remote
// 2. initialize freshly generated template
// 3. remove history from cloned template

// Assumptions:
// * `--git <url>` should only be parse in the same way as `git clone <url>` would
// * submodules should be clone by default
// * `.git` should be removed to make clear repository
// * if `<url>` is the local path on system the clone should also be done the same way as `git clone` there is `--path`
//    for different behavior

// basically we want to call:
// git clone --recurse-submodules --depth 1 --branch <branch> <url> <tmp_dir>

/// Default branch to use if not specified but required
pub const DEFAULT_BRANCH: &str = "main";

type Git2Result<T> = Result<T, git2::Error>;

struct RepoCloneBuilder<'cb> {
    builder: RepoBuilder<'cb>,
    fetch_options: FetchOptions<'cb>,
    identity: Option<PathBuf>,
    url: String,
}

impl<'cb> RepoCloneBuilder<'cb> {
    pub fn new(url: &str) -> Result<Self> {
        let mut po = ProxyOptions::new();
        po.auto();
        let mut fo = FetchOptions::new();
        fo.proxy_options(po);

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
            fetch_options: fo,
            identity: None,
            url,
        })
    }

    pub fn new_with(url: &str, branch: Option<&str>, identity_path: Option<&Path>) -> Result<Self> {
        let mut builer = Self::new(url)?;
        if let Some(branch) = branch {
            builer.set_branch(branch);
        }

        if let Some(identity_path) = identity_path {
            builer.set_identity(identity_path);
        }

        Ok(builer)
    }

    pub fn set_identity(&mut self, identity_path: &Path) {
        self.identity = Some(PathBuf::from(identity_path));
    }

    pub fn set_branch(&mut self, branch: &str) {
        self.builder.branch(branch);
    }

    fn clone(mut self, dest_path: &Path) -> Result<Repository> {
        #[cfg(not(windows))]
        {
            if self.identity.is_some() {
                if let Some(callbacks) = creds::git_ssh_credentials_callback(self.identity)? {
                    self.fetch_options.remote_callbacks(callbacks);
                } else {
                    self.fetch_options
                        .remote_callbacks(creds::git_ssh_agent_callback());
                }
            } else {
                self.fetch_options
                    .remote_callbacks(creds::git_ssh_agent_callback());
            }
        }
        #[cfg(windows)]
        {
            use crate::info;
            use console::style;

            if self.identity.is_some() {
                info!(
                    "{} {}",
                    style("HEADS UP!").bold(),
                    style("The `--identity` argument is not supported on windows, trying to use ssh-agent instead.").bold().yellow(),
                );
            }
            self.fetch_options
                .remote_callbacks(creds::git_ssh_agent_callback());
        }
        self.builder.fetch_options(self.fetch_options);
        self.builder
            .clone(&self.url, dest_path)
            .map_err(anyhow::Error::from)
    }

    pub fn clone_with_submodules(self, dest_path: &Path) -> Result<Repository> {
        self.clone(dest_path).and_then(|repo| {
            for mut sub in repo.submodules()? {
                sub.update(true, None)?;
            }

            Ok(repo)
        })
    }
}

/// Init project_dir with fresh repository on branch
///
/// Arguments:
/// - `force` - enforce a fresh git init
pub fn init(project_dir: &Path, branch: &str, force: bool) -> Git2Result<Repository> {
    fn just_init(project_dir: &Path, branch: &str) -> Git2Result<Repository> {
        let mut opts = RepositoryInitOptions::new();
        opts.bare(false);
        opts.initial_head(branch);
        Repository::init_opts(project_dir, &opts)
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
