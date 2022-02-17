//! Temporary functions to keep refactor in parts

use std::{path::Path, process::Command};

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

// `cargo` itself has a probel with dealing with git and it allow
// fallback to cli
// 1. https://doc.rust-lang.org/cargo/reference/config.html#netgit-fetch-with-cli
// 2. https://github.com/rust-lang/cargo/blob/03e24bcf67696ba35d3aa2b0dd01f97bcfdf91a7/src/cargo/sources/git/utils.rs#L875
// 3. https://github.com/rust-lang/cargo/issues/8172
pub fn clone_git_using_cmd(
    git: &str,
    branch: Option<&str>,
    identity: Option<&Path>,
) -> anyhow::Result<(TempDir, String)> {
    let temp_dir = tempfile::tempdir()?;

    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--recurse-submodules");
    if let Some(branch) = &branch {
        cmd.arg("--branch");
        cmd.arg(branch);
    }
    if let Some(identity) = identity {
        cmd.env("GIT_SSH_COMMAND", format!("ssh -i {}", identity.display()));
    }
    cmd.arg(git);
    cmd.arg(format!("{}", temp_dir.path().display()));

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(anyhow::Error::msg(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
        .context("Please check if the Git user / repository exists.");
    }
    let repo = Repository::open(temp_dir.path())?;
    let branch = get_branch_name_repo(&repo)?;

    remove_history(temp_dir.path())?;

    Ok((temp_dir, branch))
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
