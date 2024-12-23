use anyhow::{Ok, Result};
use console::style;
use log::warn;
use std::{
    fs::{copy, read_dir, remove_file, DirEntry},
    path::Path,
};

pub(crate) fn copy_files_recursively(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    overwrite: bool,
) -> Result<()> {
    let dst_path = dst.as_ref();

    for src_entry in read_dir(src.as_ref())? {
        let src_entry = src_entry?;
        let filename = src_entry.file_name().to_string_lossy().to_string();
        let entry_type = src_entry.file_type()?;

        println!("[debug] src = {src_entry:?} dst = {dst_path:?}");

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
            copy_file(src_entry, dst_path, overwrite)?;
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
fn copy_file(src_entry: DirEntry, dst: &Path, overwrite: bool) -> Result<()> {
    let filename = src_entry.file_name().to_string_lossy().to_string();
    let dst_path = dst.join(&filename);
    let src_path = src_entry.path();

    if let Some(new_filename) = filename.strip_suffix(".liquid") {
        // move the file to a new filename without the .liquid suffix, in any case
        let dst_path = dst.join(new_filename);
        safe_copy(&src_path, &dst_path, overwrite)?;
    } else {
        // if the file doesn't have a .liquid suffix, just copy it
        // possibly overwriting existing files if overwrite is true
        safe_copy(&src_path, &dst_path, overwrite)?;
    }

    Ok(())
}

fn safe_copy(src_path: &Path, dst_path: &Path, overwrite: bool) -> Result<()> {
    if dst_path.exists() && overwrite {
        remove_file(dst_path)?;
        copy(src_path, dst_path)?;
    } else if !dst_path.exists() {
        copy(src_path, dst_path)?;
    }

    Ok(())
}
