use anyhow::Result;
use std::env;

pub struct Authors {
    pub author: String,
    pub username: String,
}

/// Taken from cargo and thus (c) 2020 Cargo Developers
///
/// cf. <https://github.com/rust-lang/cargo/blob/2d5c2381e4e50484bf281fc1bfe19743aa9eb37a/src/cargo/ops/cargo_new.rs#L769-L851>
pub fn get_authors() -> Result<Authors> {
    fn get_environment_variable(variables: &[&str]) -> Option<String> {
        variables.iter().filter_map(|var| env::var(var).ok()).next()
    }

    /// Look up `key` (e.g. `user.name`) via the repo config discovered from cwd,
    /// falling back to system + global git config.
    fn read_git_config_string(key: &str) -> Option<String> {
        if let Ok(cwd) = env::current_dir() {
            if let Ok(repo) = gix::discover(&cwd) {
                if let Some(value) = repo.config_snapshot().string(key) {
                    return Some(value.to_string());
                }
            }
        }
        gix_config::File::from_globals()
            .ok()
            .and_then(|file| file.string(key).map(|v| v.to_string()))
    }

    fn discover_author() -> Result<(String, Option<String>)> {
        let name_variables = [
            "CARGO_NAME",
            "GIT_AUTHOR_NAME",
            "GIT_COMMITTER_NAME",
            "USER",
            "USERNAME",
            "NAME",
        ];
        let name = get_environment_variable(&name_variables[0..3])
            .or_else(|| read_git_config_string("user.name"))
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
            .or_else(|| read_git_config_string("user.email"))
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

    let author = match discover_author()? {
        (name, Some(email)) => Authors {
            author: format!("{name} <{email}>"),
            username: name,
        },
        (name, None) => Authors {
            author: name.clone(),
            username: name,
        },
    };

    Ok(author)
}
