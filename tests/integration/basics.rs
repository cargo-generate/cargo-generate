extern crate predicates;

use helpers::commands::*;
use helpers::project_builder::dir;
use helpers::project_templates::*;

use std::fs;

const DEFAULT_PROJECT_NAME: &str = "foobar-project";

#[test]
fn it_substitutes_projectname_in_cargo_toml() {
    let template = default_cargo_template().init_git().build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    assert!(
        dir.read(&format!("{}/Cargo.toml", cur_project_name))
            .contains(cur_project_name)
    );
}

#[test]
fn it_substitutes_date() {
    let template = cargo_template_with_copyright().init_git().build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;
    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    assert!(
        dir.read(&format!("{}/Cargo.toml", cur_project_name))
            .contains("Copyright 2018")
    );
}

#[test]
fn it_kebabcases_projectname_when_passed_to_flag() {
    let template = default_cargo_template().init_git().build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    assert!(
        dir.read(&format!("{}/Cargo.toml", cur_project_name))
            .contains(cur_project_name)
    );
}

#[test]
fn it_substitutes_cratename_in_a_rust_file() {
    let template = dir("template")
        .file(
            "main.rs",
            r#"
extern crate {{crate_name}};          
"#,
        ).init_git()
        .build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    let new_project_name: &str = "foobar_project";

    let file = dir.read(&format!("{}/main.rs", cur_project_name));
    assert!(file.contains(new_project_name));
    assert!(!file.contains(cur_project_name));
}

#[test]
fn short_commands_work() {
    let template = default_cargo_template().init_git().build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    assert!(
        dir.read(&format!("{}/Cargo.toml", cur_project_name))
            .contains(cur_project_name)
    );
}

#[test]
fn it_allows_user_defined_projectname_when_passing_force_flag() {
    let template = default_cargo_template().init_git().build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    force_generate_project(&dir, cur_project_name, &template);

    assert!(
        dir.read(&format!("{}/Cargo.toml", cur_project_name))
            .contains(cur_project_name)
    );
}

#[test]
fn it_removes_files_listed_in_genignore() {
    let template = default_cargo_template()
        .file(
            ".genignore",
            r#"deleteme.sh
*.trash
"#,
        ).file("deleteme.sh", r#"Nothing to see here"#)
        .file("deleteme.trash", r#"This is trash"#)
        .file("notme.sh", r#"I'm here!"#)
        .init_git()
        .build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    let notme_file = &format!("{}/notme.sh", cur_project_name);
    let deleteme_file = &format!("{}/deleteme.sh", cur_project_name);
    let deleteme_trash_file = &format!("{}/deleteme.trash", cur_project_name);

    assert_eq!(dir.exists(notme_file), true);
    assert_eq!(dir.exists(deleteme_file), false);
    assert_eq!(dir.exists(deleteme_trash_file), false);
}

#[test]
fn it_always_removes_genignore_file() {
    let template = default_cargo_template()
        .file(".genignore", r#"farts"#)
        .init_git()
        .build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    let genignore_file = &format!("{}/.genignore", cur_project_name);

    assert_eq!(dir.exists(genignore_file), false);
}

#[test]
fn it_does_not_remove_files_from_outside_project_dir() {
    let template = default_cargo_template()
        .file(
            ".genignore",
            r#"../dangerous.todelete.cargogeneratetests
"#,
        ).init_git()
        .build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    let dangerous_file = template
        .parent()
        .join("dangerous.todelete.cargogeneratetests");

    fs::write(&dangerous_file, "YOU BETTER NOT").expect(&format!(
        "Could not write {}",
        dangerous_file.to_str().expect("Could not read path.")
    ));

    generate_project(&dir, cur_project_name, &template);

    assert!(
        fs::metadata(&dangerous_file)
            .expect("should exist")
            .is_file()
    );
    fs::remove_file(&dangerous_file).expect("failed to clean up test file");
}

#[test]
fn errant_ignore_entry_doesnt_affect_template_files() {
    let template = default_cargo_template()
        .file(
            ".genignore",
            r#"../dangerous.todelete.cargogeneratetests
"#,
        ).file("./dangerous.todelete.cargogeneratetests", "IM FINE OK")
        .init_git()
        .build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project(&dir, cur_project_name, &template);

    assert!(
        fs::metadata(
            template
                .path()
                .join("dangerous.todelete.cargogeneratetests")
        ).expect("should exist")
        .is_file()
    );
}

#[test]
fn it_allows_a_git_branch_to_be_specified() {
    // Build and commit on mater
    let template = default_cargo_template().init_git().branch("baz").build();
    let cur_project_name: &str = DEFAULT_PROJECT_NAME;

    let dir = dir("main").build();

    generate_project_with_branch(&dir, cur_project_name, &template, "baz");

    assert!(
        dir.read(&format!("{}/Cargo.toml", cur_project_name))
            .contains(cur_project_name)
    );
}
