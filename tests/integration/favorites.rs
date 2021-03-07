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
fn it_survives_an_empty_config() {
    let empty_config = tmp_dir().build();
    let empty_config_path = empty_config.path().join("cargo-generate");
    let git_template = create_template("git-template");
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--config")
        .arg(empty_config_path)
        .arg("--name")
        .arg("foobar-project")
        .arg("--git")
        .arg(git_template.path())
        .arg("test")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains(r#"description = "git-template""#));
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
fn git_specification_overrides_favorite() {
    let git_template = create_template("git-template");
    let favorite_template = create_template("favorite-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template);
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--config")
        .arg(config_path)
        .arg("--name")
        .arg("favorite-project")
        .arg("--git")
        .arg(git_template.path())
        .arg("test")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("favorite-project/Cargo.toml")
        .contains(r#"description = "git-template""#));
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
