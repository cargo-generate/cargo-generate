extern crate predicates;

use super::helpers::*;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn simple_cargo_thing() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
authors = ["{{authors}}"]
"#,
        )
        .init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project")
    );
}
