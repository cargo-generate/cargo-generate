//! Handle `--git` and related flags

use std::path::Path;
use std::{io, ops::Sub, thread::sleep, time::Duration};

use anyhow::Result;
use log::warn;
use remove_dir_all::remove_dir_all;
pub use utils::clone_git_template_into_temp;

pub mod gitconfig;
pub mod utils;

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

/// Init `project_dir` with a fresh repository, optionally on `branch`.
///
/// * If `project_dir` (or an ancestor) already contains a repository and `force`
///   is false, the existing repo is reused unchanged.
/// * If `force` is true, we still reuse an existing repo *at* `project_dir` but
///   init a new one when only an ancestor is a repo — that matches the previous
///   `git2` behavior callers rely on.
pub fn init(project_dir: &Path, branch: Option<&str>, force: bool) -> Result<()> {
    match (gix::discover(project_dir).ok(), force) {
        (Some(_), false) => Ok(()),
        (Some(_), true) if gix::open(project_dir).is_ok() => Ok(()),
        _ => just_init(project_dir, branch),
    }
}

fn just_init(project_dir: &Path, branch: Option<&str>) -> Result<()> {
    let repo = gix::init(project_dir)?;
    if let Some(branch) = branch {
        // gix::init has no `initial_head` option — HEAD is a plain text file
        // (`ref: refs/heads/<branch>`), which is exactly what `git init -b <branch>`
        // writes. Doing it by hand keeps us clear of gix's ref-transaction surface.
        std::fs::write(
            repo.git_dir().join("HEAD"),
            format!("ref: refs/heads/{branch}\n"),
        )?;
    }
    Ok(())
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
                    warn!("Git history cleanup failed with a windows process blocking error. [Retry in {wait_for:?}]");
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
