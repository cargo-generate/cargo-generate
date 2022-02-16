//! Temporary functions to keep refactor in parts

use std::path::Path;

use git2::Repository;
use tempfile::TempDir;

use super::{remove_history, RepoCloneBuilder};
use anyhow::Context;

// this function should not be part of git module
pub fn clone_git_template_into_temp(
    git: &str,
    branch: Option<&str>,
    identity: Option<&Path>,
) -> anyhow::Result<(TempDir, String)> {
    let git_clone_dir = tempfile::tempdir()?;

    let builder = RepoCloneBuilder::new_with(git, branch, identity);

    let repo = builder
        .clone_with_submodules(git_clone_dir.path())
        .context("Please check if the Git user / repository exists.")?;
    let branch = get_branch_name_repo(&repo)?;

    // change from git repository into normal folder.
    remove_history(git_clone_dir.path())?;

    Ok((git_clone_dir, branch))
}

/// thanks to @extrawurst for pointing this out
/// <https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/branch/mod.rs#L38>
fn get_branch_name_repo(repo: &Repository) -> anyhow::Result<String> {
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
