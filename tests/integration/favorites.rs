use predicates::prelude::*;

use crate::helpers::{project::Project, project_builder::tmp_dir};

use assert_cmd::prelude::*;
use std::env;
use std::{path::PathBuf, process::Command};

fn binary() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

fn create_template(name: &str) -> Project {
    tmp_dir()
        .file(
            "Cargo.toml",
            format!(
                r#"[package]
name = "{{project-name}}"
description = "{}"
version = "0.1.0"
"#,
                name
            )
            .as_str(),
        )
        .init_git()
        .build()
}

fn create_favorite_config(name: &str, template: &Project) -> (Project, PathBuf) {
    let project = tmp_dir()
        .file(
            "cargo-generate",
            format!(
                r#"
[favorites.{name}]
description = "Favorite for the {name} template"
git = "{git}"
branch = "{branch}"
"#,
                name = name,
                git = template.path().display().to_string().escape_default(),
                branch = "main"
            )
            .as_str(),
        )
        .build();
    let path = project.path().join("cargo-generate");
    (project, path)
}

#[test]
fn favorite_with_git_becomes_subfolder() {
    let favorite_template = create_template("favorite-template");
    let git_template = create_template("git-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template);
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--config")
        .arg(config_path)
        .arg("--name")
        .arg("foobar-project")
        .arg("--git")
        .arg(git_template.path())
        .arg("test")
        .current_dir(&dir.path())
        .assert()
        .failure();
}

#[test]
fn favorite_subfolder_must_be_valid() {
    let template = tmp_dir()
        .file("Cargo.toml", "")
        .file(
            "inner/Cargo.toml",
            r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#,
        )
        .init_git()
        .build();
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("-n")
        .arg("outer")
        .arg(template.path())
        .arg("Cargo.toml")
        .current_dir(&dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("must be a valid folder").from_utf8());

    binary()
        .arg("generate")
        .arg("-n")
        .arg("outer")
        .arg(template.path())
        .arg("non-existant")
        .current_dir(&dir.path())
        .assert()
        .failure(); // Error text is OS specific

    binary()
        .arg("generate")
        .arg("-n")
        .arg("outer")
        .arg(template.path())
        .arg(dir.path().parent().unwrap())
        .current_dir(&dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid subfolder.").from_utf8());
}

#[test]
fn favorite_with_subfolder() -> anyhow::Result<()> {
    let template = tmp_dir()
        .file("Cargo.toml", "")
        .file(
            "inner/Cargo.toml",
            r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#,
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();
    binary()
        .arg("generate")
        .arg("-n")
        .arg("outer")
        .arg(template.path())
        .arg("inner")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.read("outer/Cargo.toml").contains("outer"));
    Ok(())
}

#[test]
fn it_can_use_favorites() {
    let favorite_template = create_template("favorite-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template);
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--config")
        .arg(config_path)
        .arg("--name")
        .arg("favorite-project")
        .arg("test")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("favorite-project/Cargo.toml")
        .contains(r#"description = "favorite-template""#));
}

#[test]
fn favorites_default_to_git_if_not_defined() {
    let favorite_template = create_template("favorite-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template);
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--config")
        .arg(config_path)
        .arg("--name")
        .arg("favorite-project")
        .arg("dummy")
        .current_dir(&dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(r#"status code: 404"#).from_utf8());
}
