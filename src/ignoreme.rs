use ignore::WalkBuilder;
use remove_dir_all::*;
use std::fs::remove_file;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::config::CONFIG_FILE_NAME;
pub(crate) const IGNORE_FILE_NAME: &str = ".genignore";

// We ignore the `.cargo-ok` file if one is present. This file is a somewhat
// obscure marker that cargo leaves around after extracting a tarball to
// indicate that the extraction succeeded, and is very likely not part of the
// actual template.
//
// Leaving it around is mostly harmless in the version of cargo we depend on
// (prior to 1.49, cargo could get very confused by it) but it serves no purpose
// after expanding the template, other than cluttering a user's repository.
const CARGO_OK_FILE_NAME: &str = ".cargo-ok";

/// Takes the directory path and removes the files/directories specified in the
/// `.genignore` file
/// It handles all errors internally
pub(crate) fn remove_unneeded_files(dir: &Path, verbose: bool) {
    let items = get_ignored(dir);
    remove_dir_files(items, verbose);
}

fn check_if_genignore_exists(location: &Path) -> bool {
    let mut ignore_path = PathBuf::new();
    ignore_path.push(location);
    ignore_path.push(IGNORE_FILE_NAME);
    ignore_path.exists()
}

fn get_ignored(location: &Path) -> Vec<PathBuf> {
    let default_ignored = [IGNORE_FILE_NAME, CONFIG_FILE_NAME, CARGO_OK_FILE_NAME]
        .iter()
        .map(|&file_name| location.join(file_name));
    if !check_if_genignore_exists(location) {
        return default_ignored.collect();
    }
    let all = WalkBuilder::new(location)
        .standard_filters(false)
        .build()
        .map(unwrap_path);

    let whitelisted: HashSet<_> = WalkBuilder::new(location)
        .standard_filters(false)
        .add_custom_ignore_filename(IGNORE_FILE_NAME)
        .build()
        .map(unwrap_path)
        .collect();

    default_ignored
        .chain(all.filter(|it| !whitelisted.contains(it)))
        .collect()
}

fn unwrap_path(it: Result<ignore::DirEntry, ignore::Error>) -> PathBuf {
    it.expect("Found invalid path: Aborting").into_path()
}

fn remove_dir_files(files: Vec<PathBuf>, verbose: bool) {
    for item in files.iter().filter(|file| file.exists()) {
        let ignore_message = format!("Ignoring: {}", &item.display());
        if item.is_dir() {
            remove_dir_all(&item).unwrap();
            if verbose {
                println!("{}", ignore_message);
            }
        } else if item.is_file() {
            remove_file(&item).unwrap();
            if verbose {
                println!("{}", ignore_message);
            }
        } else {
            println!(
                "The given paths are neither files nor directories! {}",
                &item.display()
            );
        }
    }
}
