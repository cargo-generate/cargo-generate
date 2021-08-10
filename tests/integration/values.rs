use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use indoc::indoc;

#[test]
fn it_accepts_template_values_from_multiple_places() {
    let template = tmp_dir()
        .file(
            "my-env-values.toml",
            indoc! {r#"
                [values]
                MY_VALUE1 = "env-file-value"
                MY_VALUE2 = "env-file-value"
                MY_VALUE3 = "env-file-value"
                MY_VALUE4 = "env-file-value"
            "#},
        )
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                MY_VALUE3 = "file-value"
                MY_VALUE4 = "file-value"
            "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value1 = "{{MY_VALUE1}}"
                value2 = "{{MY_VALUE2}}"
                value3 = "{{MY_VALUE3}}"
                value4 = "{{MY_VALUE4}}"
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
        .arg("MY_VALUE4=def-value")
        .arg("--git")
        .arg(template.path())
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .current_dir(&dir.path())
        .env(
            "CARGO_GENERATE_TEMPLATE_VALUES_FILE",
            template.path().join("my-env-values.toml"),
        )
        .env("CARGO_GENERATE_VALUE_MY_VALUE2", "env-def-value")
        .env("CARGO_GENERATE_VALUE_MY_VALUE3", "env-def-value")
        .env("CARGO_GENERATE_VALUE_MY_VALUE4", "env-def-value")
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let random_toml = dbg!(dir.read("foobar-project/random.toml"));
    assert!(random_toml.contains("value1 = \"env-file-value\""));
    assert!(random_toml.contains("value2 = \"env-def-value\""));
    assert!(random_toml.contains("value3 = \"file-value\""));
    assert!(random_toml.contains("value4 = \"def-value\""));
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
