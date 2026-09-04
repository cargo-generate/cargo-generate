//! Small helpers with no home of their own.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

/// Create a tempdir prefixed `cargo-generate` in the OS temp root.
///
/// # Errors
///
/// Returns an error when the OS temp directory cannot be created.
pub fn tmp_dir() -> std::io::Result<TempDir> {
    Builder::new().prefix("cargo-generate").tempdir()
}

/// Canonicalize `p`, first expanding a leading `~/` or `$HOME/`.
///
/// # Errors
///
/// Returns an error when `$HOME` is needed but unset, or when the
/// expanded path does not exist.
#[cfg_attr(not(feature = "git"), allow(dead_code))]
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

/// The user's home directory.
///
/// # Errors
///
/// Returns an error when `$HOME` is not set.
#[cfg_attr(not(feature = "git"), allow(dead_code))]
pub fn home() -> Result<PathBuf> {
    home::home_dir().context("$HOME was not set")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp_dir_creates_and_drops_a_directory() {
        let path = {
            let dir = tmp_dir().unwrap();
            assert!(dir.path().exists());
            assert!(dir.path().is_dir());
            assert!(dir
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .starts_with("cargo-generate"));
            dir.path().to_owned()
        };
        assert!(!path.exists(), "tempdir should clean up on drop");
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
                .map(std::path::Path::to_path_buf)
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
}
