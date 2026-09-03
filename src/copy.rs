use anyhow::{Ok, Result};
use console::style;
use log::{info, warn};
use std::{
    fs::{copy, read_dir, remove_file, File},
    io::Read,
    path::Path,
};

// Cache Directory Tagging Specification (https://bford.info/cachedir/).
// Cargo writes a `CACHEDIR.TAG` file into every `target/` build directory it
// creates, whose first 43 bytes match `CACHEDIR_TAG_SIGNATURE`. Detecting this
// marker lets us skip build directories without misidentifying user content.
const CACHEDIR_TAG_FILE: &str = "CACHEDIR.TAG";
const CACHEDIR_TAG_SIGNATURE: &[u8; 43] = b"Signature: 8a477f597d28d172789f06886806bc55";

/// Whether `dir` is a cache directory per the Cache Directory Tagging Spec —
/// i.e. contains a `CACHEDIR.TAG` file whose first 43 bytes are the spec
/// signature. Any I/O error is treated as "not a cache dir".
pub fn is_cache_dir(dir: &Path) -> bool {
    let Some(mut file) = File::open(dir.join(CACHEDIR_TAG_FILE)).ok() else {
        return false;
    };
    let mut buf = [0u8; CACHEDIR_TAG_SIGNATURE.len()];
    file.read_exact(&mut buf).is_ok() && &buf == CACHEDIR_TAG_SIGNATURE
}

pub fn copy_files_recursively(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    overwrite: bool,
) -> Result<()> {
    let dst_path = dst.as_ref();

    for src_entry in read_dir(src.as_ref())? {
        let src_entry = src_entry?;
        let filename = src_entry.file_name().to_string_lossy().to_string();
        let entry_type = src_entry.file_type()?;

        if entry_type.is_dir() {
            // we skip the .git directory
            if filename == ".git" {
                continue;
            }
            // Skip build cache directories (e.g. cargo's `target/`), walking
            // them wastes seconds to minutes on a template with a warm build.
            // See https://github.com/cargo-generate/cargo-generate/issues/1600
            if is_cache_dir(&src_entry.path()) {
                info!(
                    "Skipping cache directory (CACHEDIR.TAG): `{}`",
                    src_entry.path().display()
                );
                continue;
            }
            let dst_dir = dst_path.join(filename);
            if !dst_dir.exists() {
                std::fs::create_dir(&dst_dir)?;
            }
            copy_files_recursively(src_entry.path(), dst_dir, overwrite)?;
        } else if entry_type.is_file() {
            copy_file(&src_entry.path(), dst_path, overwrite)?;
        } else {
            // todo: maybe we better emit a warning but continue processing the other files
            warn!(
                "{} {} `{}`",
                crate::emoji::WARN,
                style("[Skipping] Symbolic links not supported")
                    .bold()
                    .red(),
                style(src_entry.path().display()).bold(),
            )
        }
    }

    Ok(())
}

/// Copy one file, overwriting an existing destination when `overwrite`
/// is set and skipping it otherwise.
///
/// Knows nothing about `.liquid`: suffixes are resolved once in
/// `crate::fetch`, on the materialized template, so every source goes
/// through the same rule.
fn copy_file(src_path: &Path, dst: &Path, overwrite: bool) -> Result<()> {
    let filename = src_path.file_name().unwrap().to_string_lossy().to_string();
    safe_copy_skip_existing(src_path, &dst.join(filename), overwrite)
}

/// Copy unless the destination exists and `overwrite` is unset, in
/// which case warn and skip rather than erroring.
fn safe_copy_skip_existing(src_path: &Path, dst_path: &Path, overwrite: bool) -> Result<()> {
    if dst_path.exists() && !overwrite {
        warn!(
            "{} `{}` {}",
            style("[Skipping] File already exists").bold().yellow(),
            style(dst_path.display()).bold(),
            style("and `--overwrite` was not passed")
        );
        return Ok(());
    }

    if dst_path.exists() && overwrite {
        remove_file(dst_path)?;
        copy(src_path, dst_path)?;
    } else if !dst_path.exists() {
        copy(src_path, dst_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_overwriting_behavior2() {
        let tmp1 = tempdir().unwrap();
        let tmp2 = tempdir().unwrap();
        let f1 = tmp1.path().join("README.md");
        std::fs::write(&f1, "FIRST README").unwrap();
        let f2 = tmp2.path().join("README.md");
        std::fs::write(&f2, "SECOND README").unwrap();

        assert!(
            safe_copy_skip_existing(f1.as_path(), f2.as_path(), false).is_ok(),
            "we do not allow overwriting if file with same name already exists without the flag set"
        );
        assert_eq!(
            std::fs::read_to_string(f2.as_path()).unwrap(),
            "SECOND README",
            "the file should not be copied"
        );
        assert!(
            safe_copy_skip_existing(f1.as_path(), f2.as_path(), true).is_ok(),
            "we do allow overwriting if file with same name already exists without the flag set"
        );
        assert_eq!(
            std::fs::read_to_string(f2.as_path()).unwrap(),
            "FIRST README"
        );
    }
}
