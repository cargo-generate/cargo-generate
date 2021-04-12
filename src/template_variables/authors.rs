use anyhow::Result;
use git2::{Config as GitConfig, Repository as GitRepository};
use std::env;

pub(crate) type Authors = String;

/// Taken from cargo and thus (c) 2020 Cargo Developers
///
/// cf. https://github.com/rust-lang/cargo/blob/2d5c2381e4e50484bf281fc1bfe19743aa9eb37a/src/cargo/ops/cargo_new.rs#L769-L851
pub(crate) fn get_authors() -> Result<Authors> {
    fn get_environment_variable(variables: &[&str]) -> Option<String> {
        variables.iter().filter_map(|var| env::var(var).ok()).next()
    }

    fn discover_author() -> Result<(String, Option<String>)> {
        let git_config = find_real_git_config();
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
                anyhow::bail!(
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
        let email = email.map(|s| {
            let mut s = s.trim();

            // In some cases emails will already have <> remove them since they
            // are already added when needed.
            if s.starts_with('<') && s.ends_with('>') {
                s = &s[1..s.len() - 1];
            }

            s.to_string()
        });

        Ok((name, email))
    }

    fn find_real_git_config() -> Option<GitConfig> {
        match env::current_dir() {
            Ok(cwd) => GitRepository::discover(cwd)
                .and_then(|repo| repo.config())
                .or_else(|_| GitConfig::open_default())
                .ok(),
            Err(_) => GitConfig::open_default().ok(),
        }
    }

    let author = match discover_author()? {
        (name, Some(email)) => format!("{} <{}>", name, email),
        (name, None) => name,
    };

    Ok(author)
}
