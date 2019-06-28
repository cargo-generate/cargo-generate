use failure;
use git2::{Config as GitConfig, Repository as GitRepository};
use quicli::prelude::*;
use std::env;

/// Taken from cargo and thus (c) 2018 Cargo Developers
///
/// cf. https://github.com/rust-lang/cargo/blob/d33c65cbd9d6f7ba1e18b2cdb85fea5a09973d3b/src/cargo/ops/cargo_new.rs#L595-L645
pub fn get_authors() -> Result<(String, String), failure::Error> {
    fn get_environment_variable(variables: &[&str]) -> Option<String> {
        variables.iter().filter_map(|var| env::var(var).ok()).next()
    }

    fn discover_author() -> Result<(String, Option<String>), failure::Error> {
        let cwd = env::current_dir()?;
        let git_config = if let Ok(repo) = GitRepository::discover(&cwd) {
            repo.config()
                .ok()
                .or_else(|| GitConfig::open_default().ok())
        } else {
            GitConfig::open_default().ok()
        };
        let git_config = git_config.as_ref();
        let name_variables = [
            "CARGO_NAME",
            "GIT_AUTHOR_NAME",
            "GIT_COMMITTER_NAME",
            "USER",
            "USERNAME",
            "NAME",
        ];
        let name = get_environment_variable(&name_variables[0..3])
            .or_else(|| git_config.and_then(|g| g.get_string("user.name").ok()))
            .or_else(|| get_environment_variable(&name_variables[3..]));

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
        let email_variables = [
            "CARGO_EMAIL",
            "GIT_AUTHOR_EMAIL",
            "GIT_COMMITTER_EMAIL",
            "EMAIL",
        ];
        let email = get_environment_variable(&email_variables[0..3])
            .or_else(|| git_config.and_then(|g| g.get_string("user.email").ok()))
            .or_else(|| get_environment_variable(&email_variables[3..]));

        let name = name.trim().to_string();
        let email = email.map(|s| s.trim().to_string());

        Ok((name, email))
    }

    let author = discover_author()?;
    let username = author.0.clone();

    let authors_field = match author.1 {
        Some(email) => format!("{} <{}>", author.0, email),
        None => author.0,
    };

    Ok((username, authors_field))
}
