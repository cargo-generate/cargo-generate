use git2::{
    build::CheckoutBuilder, build::RepoBuilder, Repository as GitRepository, RepositoryInitOptions, SubmoduleUpdateOptions
};
use quicli::prelude::*;
use remove_dir_all::remove_dir_all;
use std::path::PathBuf;
use Args;

pub fn create(project_dir: &PathBuf, args: Args) -> Result<GitRepository> {
    let mut rb = RepoBuilder::new();
    rb.bare(false).with_checkout(CheckoutBuilder::new());

    if let Some(ref branch) = args.branch {
        rb.branch(branch);
    }

    Ok(rb.clone(&args.git, &project_dir)?)
}

pub fn load_submodules(git_repository: &GitRepository) -> Result<()> {
    let submodules = git_repository.submodules()?;
    if submodules.len() == 0 {
        return Ok(())
    }

    // Init submodule options
    let mut submodule_opts = SubmoduleUpdateOptions::new();
    submodule_opts.allow_fetch(true).checkout(CheckoutBuilder::new());

    // Init and load each submodule
    for mut submodule in submodules {
        submodule.update(true, Some(&mut submodule_opts))?;
    };

    Ok(())
}

pub fn remove_history(project_dir: &PathBuf) -> Result<()> {
    Ok(remove_dir_all(project_dir.join(".git")).context("Error cleaning up cloned template")?)
}

pub fn init(project_dir: &PathBuf) -> Result<GitRepository> {
    Ok(
        GitRepository::init_opts(project_dir, RepositoryInitOptions::new().bare(false))
            .context("Couldn't init new repository")?,
    )
}
