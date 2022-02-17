use crate::git::identity_path::IdentityPath;
use crate::git::utils::home;
use crate::info;
use anyhow::Result;
use console::style;
use git2::{Cred, RemoteCallbacks};
use std::path::PathBuf;

/// takes care of `~/` paths, defaults to `$HOME/.ssh/id_rsa` and resolves symlinks.
fn get_private_key_path(identity: Option<PathBuf>) -> Result<IdentityPath> {
    let private_key = identity.unwrap_or(home()?.join(".ssh/id_rsa"));
    private_key.try_into()
}

pub fn git_ssh_credentials_callback<'a>(identity: Option<PathBuf>) -> Result<RemoteCallbacks<'a>> {
    let private_key = get_private_key_path(identity)?;
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
    Ok(cb)
}
