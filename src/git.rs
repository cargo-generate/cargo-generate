use git2::{Repository as GitRepository, RepositoryInitOptions};
use quicli::prelude::*;
use remove_dir_all::remove_dir_all;
use std::path::PathBuf;
use tempfile::Builder;
use upstream::core::GitReference;
use upstream::sources::git::GitRemote;
use upstream::util::config::Config;
use url::{ParseError, Url};

pub struct GitConfig {
    remote: Url,
    branch: GitReference,
}

impl GitConfig {
    pub fn new(git: String, branch: Option<String>) -> Result<Self> {
        let remote = match Url::parse(&git) {
            Ok(u) => u,
            Err(ParseError::RelativeUrlWithoutBase) => {
                let rel = "file://".to_string() + &git;
                Url::parse(&rel)?
            }
            Err(_) => return Err(format_err!("Failed parsing git remote: {}", &git)),
        };

        Ok(GitConfig {
            remote: remote,
            branch: GitReference::Branch(branch.unwrap_or("master".to_string())),
        })
    }
}

pub fn create(project_dir: &PathBuf, args: GitConfig) -> Result<()> {
    let temp = Builder::new()
        .prefix(project_dir.to_str().unwrap_or("cargo-generate"))
        .tempdir()?;
    let config = Config::default()?;
    let remote = GitRemote::new(&args.remote);
    let (db, rev) = remote.checkout(&temp.path(), &args.branch, &config)?;

    // This clones the remote and handles all the submodules
    db.copy_to(rev, project_dir.as_path(), &config)?;
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
