//! Cloning a template repository into a temp directory.

use std::path::Path;

use super::gix::RepoCloneBuilder;
use crate::fetch::FetchedSource;

/// Clone `git_url` into a fresh temp directory.
///
/// # Errors
///
/// Returns an error when the temp directory cannot be created or the
/// clone fails — unreachable ref, bad ssh identity, network, and so on.
pub fn clone_git_template_into_temp(
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
    identity: Option<&Path>,
    gitconfig: Option<&Path>,
    skip_submodules: bool,
) -> anyhow::Result<FetchedSource> {
    let git_clone_dir = crate::utils::tmp_dir()?;

    let branch = RepoCloneBuilder::new(git_url)
        .with_branch(branch)
        .with_ssh_identity(identity)?
        .with_submodules(!skip_submodules)
        .with_gitconfig(gitconfig)?
        .with_destination(git_clone_dir.path())?
        .with_tag(tag)
        .with_revision(revision)
        .build()?
        .do_clone()?;

    Ok(FetchedSource::new(git_clone_dir, Some(branch)))
}
