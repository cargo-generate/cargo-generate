use anyhow::{bail, Ok, Result};
use console::style;
use log::{debug, warn};
use std::{
    fs::{copy, read_dir, remove_file},
    path::Path,
};

const LIQUID_SUFFIX: &str = ".liquid";

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

/// move a file from src to dst, possibly overwriting existing files if overwrite is true
/// if the file has a .liquid suffix, the suffix will be removed in the destination, and overwritten if existing
fn copy_file(src_path: &Path, dst: &Path, overwrite: bool) -> Result<()> {
    let filename = src_path.file_name().unwrap().to_string_lossy().to_string();
    let dst_path = dst.join(&filename);
    let mut overwrite = overwrite;

    if let Some(new_filename) = filename.strip_suffix(LIQUID_SUFFIX) {
        if src_path.with_file_name(new_filename).exists() {
            // if there is a file without the .liquid suffix, we want to set overwrite to true
            // so that this liquid file takes precedence over the existing file
            debug!("A non-liquid file exists for {filename}, overwriting it with the liquid file");
            overwrite = true;
        }

        // move the file to a new filename without the .liquid suffix, in any case
        let dst_path = dst.join(new_filename);
        safe_copy(src_path, &dst_path, overwrite)?;
    } else if src_path
        .with_file_name(format!("{filename}{LIQUID_SUFFIX}"))
        .exists()
    {
        // there is a liquid file for this non-liquid file,
        // so we skip this one, so that the liquid file takes precedence
        debug!("A liquid file exists for {filename}, skipping the non-liquid file");
        return Ok(());
    } else {
        // if the file doesn't have a .liquid suffix, just copy it
        // possibly overwriting existing files if overwrite is true
        safe_copy(src_path, &dst_path, overwrite)?;
    }

    Ok(())
}

fn safe_copy(src_path: &Path, dst_path: &Path, overwrite: bool) -> Result<()> {
    if dst_path.exists() && !overwrite {
        bail!(
            "{} {} `{}` {}",
            crate::emoji::ERROR,
            style("File already exists").bold().red(),
            style(dst_path.display()).bold(),
            style("and `--overwrite` was not passed")
        )
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
    fn test_overwriting_behavior() {
        let tmp = tempdir().unwrap();
        let f1 = tmp.path().join("README.md");
        std::fs::write(&f1, "A README").unwrap();
        let f2 = tmp.path().join("README.md.liquid");
        std::fs::write(&f2, "A README liquid file").unwrap();

        assert!(
            safe_copy(f1.as_path(), f2.as_path(), false).is_err(),
            "we do not allow overwriting without the flag set"
        );
        assert!(
            safe_copy(f1.as_path(), f2.as_path(), true).is_ok(),
            "we do allow overwriting with the flag set"
        );
        assert_eq!(std::fs::read_to_string(f2.as_path()).unwrap(), "A README");
    }

    #[test]
    fn test_special_liquid_file_handling() {
        let tmp = tempdir().unwrap();
        let f1 = tmp.path().join("README.md");
        std::fs::write(&f1, "A README").unwrap();
        let f2 = tmp.path().join("README.md.liquid");
        std::fs::write(&f2, "A README liquid file").unwrap();

        let tmp2 = tempdir().unwrap();

        // copy the non liquid file first, should not copy anything
        copy_file(f1.as_path(), tmp2.path(), false).unwrap();
        assert!(
            !tmp2.path().join("README.md").exists(),
            "the file should not be copied"
        );

        // copy the liquid file, should copy the liquid file and remove the .liquid suffix
        copy_file(f2.as_path(), tmp2.path(), false).unwrap();
        assert!(
            tmp2.path().join("README.md").exists(),
            "the file should be copied and the .liquid suffix removed"
        );
        assert_eq!(
            std::fs::read_to_string(tmp2.path().join("README.md")).unwrap(),
            "A README liquid file"
        );

        // copy the liquid file, while the same file without the .liquid suffix exists,
        // should overwrite the existing file, as if the overwrite flag was set
        let f4 = tmp2.path().join("README.md");
        std::fs::write(&f4, "Existing file, should be overwritten").unwrap();
        assert!(
            copy_file(f2.as_path(), tmp2.path(), false).is_ok(),
            "the file should be copied"
        );
        assert_eq!(
            std::fs::read_to_string(tmp2.path().join("README.md")).unwrap(),
            "A README liquid file"
        );
    }
}
