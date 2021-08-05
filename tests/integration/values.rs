use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use indoc::indoc;

#[test]
fn it_accepts_template_values_file() {
    let template = tmp_dir()
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                my_value = "content of my-value"
            "#},
        )
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "{{my_value}}"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--name")
        .arg("foobar-project")
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .arg(template.path())
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of my-value"));
}

#[test]
fn it_accepts_values_via_long_option() {
    let template = tmp_dir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "{{my_value}}"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--name")
        .arg("foobar-project")
        .arg("--define")
        .arg(r#"my_value="content of my-value""#)
        .arg(template.path())
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of my-value"));
}

#[test]
fn it_accepts_values_via_short_option() {
    let template = tmp_dir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "{{my_value}}"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--name")
        .arg("foobar-project")
        .arg("-d")
        .arg(r#"my_value="content of my-value""#)
        .arg(template.path())
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of my-value"));
}

#[test]
fn cli_value_overrides_others() {
    let template = tmp_dir()
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                my_value = "content of file-value"
            "#},
        )
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "{{my_value}}"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--name")
        .arg("foobar-project")
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .arg("-d")
        .arg(r#"my_value="content of cli-value""#)
        .arg(template.path())
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of cli-value"));
}
