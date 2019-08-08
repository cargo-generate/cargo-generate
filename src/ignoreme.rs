use ignore::WalkBuilder;
use remove_dir_all::*;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::remove_file;
use std::path::{Path, PathBuf};

use crate::config::CONFIG_FILE_NAME;
pub const IGNORE_FILE_NAME: &str = ".genignore";

///takes the directory path and removes the files/directories specified in the
/// `.genignore` file
/// It handles all errors internally
pub fn remove_uneeded_files(dir: &PathBuf) {
    if check_if_genignore_exists(dir) {
        let items = get_ignored(dir);
        remove_dir_files(items);
    }
}

fn check_if_genignore_exists(location: &PathBuf) -> bool {
    let mut ignore_path = PathBuf::new();
    ignore_path.push(location);
    ignore_path.push(IGNORE_FILE_NAME);
    ignore_path.exists()
}

fn get_ignored(location: &PathBuf) -> Vec<PathBuf> {
    let ignored = WalkBuilder::new(location)
        .standard_filters(false)
        .add_custom_ignore_filename(OsStr::new(IGNORE_FILE_NAME))
        .build();

    let all = WalkBuilder::new(location).standard_filters(false).build();

    let mut all_set = HashSet::new();
    let mut ign_set = HashSet::new();
    let mut output = vec![
        Path::new(location).join(IGNORE_FILE_NAME),
        Path::new(location).join(CONFIG_FILE_NAME),
    ];

    for x in all {
        all_set.insert(x.expect("Found invalid path: Aborting").path().to_owned());
    }
    for x in ignored {
        ign_set.insert(x.expect("Found invalid path: Aborting").path().to_owned());
    }
    for x in all_set.difference(&ign_set) {
        output.push(x.to_owned());
    }
    output
}

fn remove_dir_files(files: Vec<PathBuf>) {
    for item in files {
        if item.is_dir() {
            remove_dir_all(&item).unwrap();
            println!("Removed: {:?}", &item)
        } else if item.is_file() {
            remove_file(&item).unwrap();
            println!("Removed: {:?}", &item)
        } else {
            println!(
                "The given paths are neither files nor directories! {:?}",
                &item
            );
        }
    }
}
