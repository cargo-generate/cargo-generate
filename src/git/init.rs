//! Git `init` for the finalized project. Requires the `git` feature.

use std::path::Path;

use anyhow::Result;

/// Init `project_dir` with a fresh repository, optionally on `branch`.
///
/// * If `project_dir` (or an ancestor) already contains a repository
///   and `force` is false, the existing repo is reused unchanged.
/// * If `force` is true, we still reuse an existing repo *at*
///   `project_dir` but init a new one when only an ancestor is a
///   repo — matches previous `git2` behavior callers rely on.
///
/// # Errors
///
/// Returns an error when the repository cannot be initialized.
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
