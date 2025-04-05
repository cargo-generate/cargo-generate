//! Handle `--git` and related flags

use std::path::Path;
use std::{io, ops::Sub, thread::sleep, time::Duration};

use anyhow::Result;
use git2::{Repository, RepositoryInitOptions};
use log::warn;
use remove_dir_all::remove_dir_all;
pub use utils::clone_git_template_into_temp;

mod clone_tool;
pub(super) mod gitconfig;
pub(super) mod utils;

pub use utils::{tmp_dir, try_get_branch_from_path};

// cargo-generate (as application) want from git module:
// 1. cloning remote
// 2. initialize freshly generated template
// 3. remove history from cloned template

// Assumptions:
// * `--git <url>` should only be parse in the same way as `git clone <url>` would
// * submodules are cloned by default, but can be skipped by `--skip-submodules`.
// * `.git` should be removed to make clear repository
// * if `<url>` is the local path on system the clone should also be done the same way as `git clone` there is `--path`
//    for different behavior

// basically we want to call:
// git clone --recurse-submodules --depth 1 --branch <branch> <url> <tmp_dir>
// with --recurse-submodules being optional.

type Git2Result<T> = Result<T, git2::Error>;

/// Init project_dir with fresh repository on branch
///
/// Arguments:
/// - `force` - enforce a fresh git init
pub fn init(project_dir: &Path, branch: Option<&str>, force: bool) -> Git2Result<Repository> {
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

fn just_init(project_dir: &Path, branch: Option<&str>) -> Git2Result<Repository> {
    let mut opts = RepositoryInitOptions::new();
    opts.bare(false);
    if let Some(branch) = branch {
        opts.initial_head(branch);
    }
    Repository::init_opts(project_dir, &opts)
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
