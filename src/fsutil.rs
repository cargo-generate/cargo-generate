use std::{fs, path::Path};

use anyhow::{bail, Result};
use console::style;

pub(crate) fn copy_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src_type = fs::metadata(&src)?.file_type();
    if src_type.is_dir() {
        copy_dir_all(src, dst)?;
    } else if src_type.is_file() {
        fs::create_dir_all(&dst)?;
        fs::copy(&src, dst.as_ref().join(src.as_ref().file_name().unwrap()))?;
    } else {
        bail!(
            "{} {}",
            crate::emoji::WARN,
            style("Symbolic links not supported").bold().red(),
        )
    }
    Ok(())
}

pub(crate) fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            if entry.file_name() != ".git" {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        } else if ty.is_file() {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            bail!(
                "{} {}",
                crate::emoji::WARN,
                style("Symbolic links not supported").bold().red(),
            )
        }
    }
    Ok(())
}
