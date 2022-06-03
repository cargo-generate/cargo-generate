use anyhow::Context;
use anyhow::Result;
use std::path::{Path, PathBuf};

use git2::Repository;
use tempfile::TempDir;

use super::RepoCloneBuilder;

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
    dirs::home_dir().context("$HOME was not set")
}

// clone git reposiotry into temp using libgit2
pub fn clone_git_template_into_temp(
    git: &str,
    branch: Option<&str>,
    identity: Option<&Path>,
) -> anyhow::Result<(TempDir, String)> {
    let git_clone_dir = tempfile::tempdir()?;

    let builder = RepoCloneBuilder::new_with(git, branch, identity)?;

    let repo = builder
        .clone_with_submodules(git_clone_dir.path())
        .context("Please check if the Git user / repository exists.")?;
    let branch = get_branch_name_repo(&repo)?;

    // change from git repository into normal folder.
    // remove_history(git_clone_dir.path())?;

    Ok((git_clone_dir, branch))
}

/// thanks to @extrawurst for pointing this out
/// <https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/branch/mod.rs#L38>
fn get_branch_name_repo(repo: &Repository) -> anyhow::Result<String> {
    let iter = repo.branches(None)?;

    for b in iter {
        let b = b?;

        if b.0.is_head() {
            let name = b.0.name()?.unwrap_or("");
            return Ok(name.into());
        }
    }

    anyhow::bail!("A repo has no Head")
}

#[test]
fn should_canonicalize() {
    #[cfg(target_os = "macos")]
    {
        assert!(canonicalize_path(&PathBuf::from("../"))
            .unwrap()
            .starts_with("/Users/"));

        assert!(canonicalize_path(&PathBuf::from("$HOME/"))
            .unwrap()
            .starts_with("/Users/"));
    }
    #[cfg(target_os = "linux")]
    assert_eq!(
        canonicalize_path(&PathBuf::from("../")).ok(),
        std::env::current_dir()
            .unwrap()
            .parent()
            .map(|p| p.to_path_buf())
    );
    #[cfg(windows)]
    assert!(canonicalize_path(&PathBuf::from("../"))
        .unwrap()
        // not a bug, a feature:
        // https://stackoverflow.com/questions/41233684/why-does-my-canonicalized-path-get-prefixed-with
        .to_str()
        .unwrap()
        .starts_with("\\\\?\\"));
}
