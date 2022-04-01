use crate::git::utils::home;
use crate::git::{identity_path::IdentityPath, utils::canonicalize_path};
use crate::info;
use anyhow::Result;
use console::style;
use git2::{Cred, RemoteCallbacks};
use std::path::PathBuf;

// Ok(None) is returned when identity was None and .ssh/id_rsa was not present
pub fn git_ssh_credentials_callback<'a>(
    identity: Option<PathBuf>,
) -> Result<Option<RemoteCallbacks<'a>>> {
    let private_key = if let Some(identity) = identity {
        let identity = canonicalize_path(identity)?;
        IdentityPath::try_from(identity)?
    } else {
        // if .ssh/id_rsa not exist its not error
        let default_ssh_key = home()?.join(".ssh/id_rsa");
        match IdentityPath::try_from(default_ssh_key) {
            Ok(v) => v,
            Err(_e) => return Ok(None),
        }
    };

    info!(
        "{} `{}` {}",
        style("Using private key:").bold(),
        style(format!("{}", &private_key)).bold().yellow(),
        style("for git-ssh checkout").bold()
    );
    let mut cb = RemoteCallbacks::new();
    cb.credentials(
        move |_url, username_from_url: Option<&str>, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                private_key.as_ref(),
                None,
            )
        },
    );
    Ok(Some(cb))
}
