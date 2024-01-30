use super::gix_exploration::RepoCloneBuilderImpl;
use super::BranchName;
use crate::copy_dir_all;
use crate::utils::tmp_dir;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub fn clone_local_path_as_if_it_was_a_repo(
    src: &PathBuf,
    dest: &Path,
    overwrite: bool,
) -> Result<Option<BranchName>> {
    copy_dir_all(src, dest, false)?;

    let repo = gix::discover(".")?;
    
    todo!("cloning a local path is not yet supported")
}

pub fn clone_git_template_into_temp(
    git: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
    identity: Option<&Path>,
) -> Result<(TempDir, String)> {
    let git_clone_dir = tmp_dir()?;

    let builder = RepoCloneBuilderImpl::new(git)?
        .with_branch(branch)
        .with_identity_file(identity);

    let branch = builder
        .checkout(git_clone_dir.path())
        .context("Please check if the Git user / repository exists.")?;

    if let Some(spec) = tag.or(revision) {
        todo!("tag or revision or is not yet supported");
        //     let (object, reference) = repo.revparse_ext(spec)?;
        //     repo.checkout_tree(&object, None)?;
        //     reference.map_or_else(
        //         || repo.set_head_detached(object.id()),
        //         |gref| repo.set_head(gref.name().unwrap()),
        //     )?
    }

    Ok((git_clone_dir, branch))
}
