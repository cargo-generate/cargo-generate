use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use std::env;
use std::process::Command;

fn binary() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn it_uses_existing_directory_when_passing_bare_flag() {
    let template = tmp_dir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file("README.md", r#"A wonderful project"#)
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--bare")
        .current_dir(&dir.path())
        .assert()
        .success();

    assert!(!dir.exists("foobar-project"));
    assert!(dir.exists("Cargo.toml"));
    assert!(dir.exists("README.md"));
}

#[test]
fn it_doesnt_write_over_existing_git_repo_when_passing_bare_flag() {
    let template = tmp_dir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tmp_dir()
        .file("README.md", r#"A wonderful project"#)
        .init_git()
        .build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--bare")
        .current_dir(&dir.path())
        .assert()
        .success();

    Command::new("git")
        .arg("log")
        .current_dir(&dir.path())
        .assert()
        .success();
}
