use crate::git::identity_path::IdentityPath;
use crate::git::utils::home;
use crate::info;
use anyhow::Result;
use console::style;
use git2::{Cred, RemoteCallbacks};
use std::path::PathBuf;

// Ok(None) is returned when identity was None and .ssh/id_rsa was not present
pub fn git_ssh_credentials_callback<'a>(
    identity: Option<PathBuf>,
) -> Result<Option<RemoteCallbacks<'a>>> {
    let private_key = identity.or_else(|| home().map(|h| h.join(".ssh/id_rsa")).ok());

    if private_key.is_none() {
        return Ok(None);
    }

    let private_key = IdentityPath::try_from(private_key.unwrap())?;

    let mut cb = RemoteCallbacks::new();
    cb.credentials(
        move |_url, username_from_url: Option<&str>, _allowed_types| {
            info!(
                "{} `{}` {}",
                style("Using private key:").bold(),
                style(format!("{}", &private_key)).bold().yellow(),
                style("for git-ssh checkout").bold()
            );
            let username = username_from_url.unwrap_or("git");
            Cred::ssh_key(username, None, private_key.as_ref(), None)
        },
    );
    Ok(Some(cb))
}

pub fn git_ssh_agent_callback<'a>() -> RemoteCallbacks<'a> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(
        move |_url, username_from_url: Option<&str>, _allowed_types| {
            let username = username_from_url.unwrap_or("git"); 
            Ok(Cred::ssh_key_from_agent(username)
                .unwrap_or_else(|e| panic!("There was a problem talking to your ssh-agent: {:?}\n\ncheck our Q&A thread: https://github.com/cargo-generate/cargo-generate/discussions/653", e)))
    });

    cb
}
