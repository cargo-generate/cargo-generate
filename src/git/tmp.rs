//! Tempdir helper. Always compiled — every fetch path needs it,
//! regardless of the `git` feature.

use tempfile::{Builder, TempDir};

/// Create a tempdir prefixed `cargo-generate` in the OS temp root.
///
/// # Errors
///
/// Returns an error when the OS temp directory cannot be created.
pub fn tmp_dir() -> std::io::Result<TempDir> {
    Builder::new().prefix("cargo-generate").tempdir()
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
}
