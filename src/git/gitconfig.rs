use crate::git::utils::home;
use anyhow::Context;
use anyhow::Result;
use gix_config::{File as GitConfigParser, Source};
use std::path::{Path, PathBuf};

pub fn find_gitconfig() -> Result<Option<PathBuf>> {
    let gitconfig = home().map(|home| home.join(".gitconfig"))?;
    if gitconfig.exists() {
        return Ok(Some(gitconfig));
    }

    Ok(None)
}

/// trades urls, to replace a given repo remote url with the right on based
/// on the `[url]` section in the `~/.gitconfig`
pub fn resolve_instead_url(
    remote: impl AsRef<str>,
    gitconfig: impl AsRef<Path>,
) -> Result<Option<String>> {
    let gitconfig = gitconfig.as_ref();
    let remote = remote.as_ref().to_string();
    let config = GitConfigParser::from_path_no_includes(gitconfig, Source::User)
        .context("Cannot read or parse .gitconfig")?;
    let x = config.sections_by_name("url").and_then(|iter| {
        iter.map(|section| {
            let head = section.header();
            let body = section.body();
            let url = head.subsection_name();
            let instead_of = body
                .value("insteadOf")
                .map(|x| std::str::from_utf8(&x[..]).unwrap().to_owned());
            (instead_of, url)
        })
        .filter(|(old, new)| new.is_some() && old.is_some())
        .find_map(|(old, new)| {
            let old = old.unwrap();
            let new = new.unwrap().to_string();
            remote
                .starts_with(old.as_str())
                .then(|| remote.replace(old.as_str(), new.as_str()))
        })
    });

    Ok(x)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_resolve_instead_url() {
        let sample_config = r#"
[url "ssh://git@github.com:"]
    insteadOf = https://github.com/
"#;
        let where_gitconfig_lives = tempfile::tempdir().unwrap();
        let gitconfig = where_gitconfig_lives.path().join(".gitconfig");
        std::fs::write(&gitconfig, sample_config).unwrap();

        // SSH, aka git@github.com: or ssh://git@github.com/
        let x = resolve_instead_url("https://github.com/foo/bar.git", &gitconfig).unwrap();
        assert_eq!(x.unwrap().as_str(), "ssh://git@github.com:foo/bar.git")
    }
}
