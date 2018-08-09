use ignore::WalkBuilder;
use remove_dir_all::*;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::remove_file;
use std::path::PathBuf;

//FIXME: function returns Result
pub fn remove_uneeded_files(dir: &PathBuf) {
    match get_ignored(dir) {
        Some(items) => remove_dir_files(items),
        None => println!("Nothing to remove!"),
    }
}

fn get_ignored(location: &PathBuf) -> Option<Vec<PathBuf>> {
    let ignored = WalkBuilder::new(location)
        .standard_filters(false)
        .add_custom_ignore_filename(OsStr::new(".genignore"))
        .build()
        .into_iter()
        .map(|x| x.unwrap());

    let all = WalkBuilder::new(location)
        .standard_filters(false)
        .build()
        .into_iter()
        .map(|x| x.unwrap());

    let mut all_set = HashSet::new();
    let mut ign_set = HashSet::new();
    let mut output = vec![];

    for x in all {
        all_set.insert(x.path().to_owned());
    }
    for x in ignored {
        ign_set.insert(x.path().to_owned());
    }
    for x in all_set.difference(&ign_set) {
        output.push(x.to_owned());
    }
    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

//FIXME:
fn sanititze(items: Vec<PathBuf>) -> Option<Vec<PathBuf>> {
    unimplemented!()
}

//FIXME:
fn remove_dir_files(ignore: Vec<PathBuf>) {
    let sanitized_files = ignore; //sanititze(ignore);

    for item in sanitized_files {
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
