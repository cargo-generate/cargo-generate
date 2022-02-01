use anyhow::Result;
use console::style;
use core::convert::{AsRef, TryFrom};
use core::result::Result::Ok;
use std::borrow::Cow;
use std::path::PathBuf;

use crate::git::utils::{canonicalize_path, determine_repo_kind, resolve_instead_url, RepoKind};
use crate::info;

/// url and gitconfig content
pub struct GitUrlAndConfig<'a>(pub Cow<'a, str>, pub PathBuf);

impl<'a> TryFrom<GitUrlAndConfig<'a>> for GitRemote<'a> {
    type Error = anyhow::Error;

    fn try_from(value: GitUrlAndConfig<'a>) -> Result<Self, Self::Error> {
        let r = value.0.as_ref();
        let url = if let Ok(Some(url)) = resolve_instead_url(r, &value.1) {
            info!(
                "Using gitconfig `{}` for {:?} -> {:?}",
                style("insteadOf").bold(),
                style(r).bold().yellow(),
                style(&url).bold().yellow()
            );
            url
        } else {
            r.to_owned()
        };

        url.try_into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitRemote<'a> {
    url: Cow<'a, str>,
    kind: RepoKind,
}

impl<'a> AsRef<str> for GitRemote<'a> {
    fn as_ref(&self) -> &str {
        self.url.as_ref()
    }
}

impl<'a> TryFrom<Cow<'a, str>> for GitRemote<'a> {
    type Error = anyhow::Error;

    fn try_from(url: Cow<'a, str>) -> Result<Self, Self::Error> {
        let url = url.as_ref();
        let (remote, kind) = match determine_repo_kind(url) {
            RepoKind::Invalid => anyhow::bail!("Invalid git remote '{}'", url),
            RepoKind::LocalFolder => {
                let full_path = canonicalize_path(&url)?;
                if !full_path.exists() {
                    anyhow::bail!("The given git remote {:?} does not exist.", url);
                }
                (full_path.display().to_string(), RepoKind::LocalFolder)
            }
            k => (url.to_string(), k),
        };
        Ok(Self {
            url: Cow::from(remote),
            kind,
        })
    }
}

impl<'a> TryFrom<String> for GitRemote<'a> {
    type Error = anyhow::Error;

    fn try_from(url: String) -> Result<Self, Self::Error> {
        Cow::from(url).try_into()
    }
}

impl<'a> TryFrom<&'a str> for GitRemote<'a> {
    type Error = anyhow::Error;

    fn try_from(url: &'a str) -> Result<Self, Self::Error> {
        Cow::from(url).try_into()
    }
}

impl<'a> AsRef<RepoKind> for GitRemote<'a> {
    fn as_ref(&self) -> &RepoKind {
        &self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_provide_a_remove() {
        let sample_config = r#"
[url "ssh://git@github.com:"]
    insteadOf = https://github.com/
"#;
        let where_gitconfig_lives = tempfile::tempdir().unwrap();
        let gitconfig = where_gitconfig_lives.path().join(".gitconfig");
        std::fs::write(&gitconfig, sample_config).unwrap();

        let remote = GitUrlAndConfig(Cow::from("https://github.com/sassman/t-rec-rs"), gitconfig);
        let remote: GitRemote = remote.try_into().unwrap();
        let remote: &str = remote.as_ref();

        assert_eq!(remote, "ssh://git@github.com:sassman/t-rec-rs");
    }
}
