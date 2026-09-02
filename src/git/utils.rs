use anyhow::Context;
use anyhow::Result;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::gix::RepoCloneBuilder;

/// deals with `~/` and `$HOME/` prefixes
pub fn canonicalize_path(p: impl AsRef<Path>) -> Result<PathBuf> {
    let p = p.as_ref();
    let p = if p.starts_with("~/") {
        home()?.join(p.strip_prefix("~/")?)
    } else if p.starts_with("$HOME/") {
        home()?.join(p.strip_prefix("$HOME/")?)
    } else {
        p.to_path_buf()
    };

    p.canonicalize()
        .with_context(|| format!("path does not exist: {}", p.display()))
}

/// home path wrapper
pub fn home() -> Result<PathBuf> {
    home::home_dir().context("$HOME was not set")
}

// clone git repository into temp using gix
pub fn clone_git_template_into_temp(
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
    identity: Option<&Path>,
    gitconfig: Option<&Path>,
    skip_submodules: bool,
) -> anyhow::Result<(TempDir, Option<String>)> {
    let git_clone_dir = super::tmp_dir()?;

    let branch = RepoCloneBuilder::new(git_url)
        .with_branch(branch)
        .with_ssh_identity(identity)?
        .with_submodules(!skip_submodules)
        .with_gitconfig(gitconfig)?
        .with_destination(git_clone_dir.path())?
        .with_tag(tag)
        .with_revision(revision)
        .build()?
        .do_clone()?;

    Ok((git_clone_dir, Some(branch)))
}

#[test]
fn should_canonicalize() {
    #[cfg(target_os = "macos")]
    {
        assert!(canonicalize_path(PathBuf::from("../"))
            .unwrap()
            .starts_with("/Users/"));

        assert!(canonicalize_path(PathBuf::from("$HOME/"))
            .unwrap()
            .starts_with("/Users/"));
    }
    #[cfg(target_os = "linux")]
    assert_eq!(
        canonicalize_path(PathBuf::from("../")).ok(),
        std::env::current_dir()
            .unwrap()
            .parent()
            .map(|p| p.to_path_buf())
    );
    #[cfg(windows)]
    assert!(canonicalize_path(PathBuf::from("../"))
        .unwrap()
        // not a bug, a feature:
        // https://stackoverflow.com/questions/41233684/why-does-my-canonicalized-path-get-prefixed-with
        .to_str()
        .unwrap()
        .starts_with("\\\\?\\"));
}
