use failure;
use git2::{Config as GitConfig, Repository as GitRepository};
use quicli::prelude::*;
use std::env;

pub fn get_username() -> Result<String, failure::Error> {
    let cwd = env::current_dir()?;
    let git_config = if let Ok(repo) = GitRepository::discover(&cwd) {
        repo.config()
            .ok()
            .or_else(|| GitConfig::open_default().ok())
    } else {
        GitConfig::open_default().ok()
    };

    let git_config = git_config.as_ref();
    let config_vars = ["CARGO_NAME", "GIT_AUTHOR_NAME", "GIT_COMMITTER_NAME"]
        .iter()
        .filter_map(|var| env::var(var).ok())
        .next();

    let system_vars = ["USER", "USERNAME", "NAME"]
        .iter()
        .filter_map(|var| env::var(var).ok())
        .next();

    let name = config_vars
        .or_else(|| git_config.and_then(|g| g.get_string("user.name").ok()))
        .or_else(|| system_vars);

    let name = match name {
        Some(name) => name,
        None => {
            let username_var = if cfg!(windows) { "USERNAME" } else { "USER" };
            bail!(
                "could not determine the current user, please set ${}",
                username_var
            )
        }
    };

    Ok(name)
}
