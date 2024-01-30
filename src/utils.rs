use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// creates a `TempDir`
pub fn tmp_dir() -> std::io::Result<TempDir> {
    tempfile::Builder::new().prefix("cargo-generate").tempdir()
}

/// home path wrapper
pub fn home() -> Result<PathBuf> {
    home::home_dir().context("$HOME was not set")
}

/// deals with `~/` and `$HOME/` prefixes
/// might not work on windows
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
