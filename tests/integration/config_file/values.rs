use crate::helpers::prelude::*;
#[test]
fn it_accepts_default_template_values() {
    let config_dir = tempdir()
        .file(
            "cargo-generate",
            indoc! {r#"
                [values]
                my_value = "default value"
                "#},
        )
        .build();

    let template_dir = tempdir()
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{my_value}}"
            "#},
        )
        .init_git()
        .build();

    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_dir.path().join("cargo-generate"))
        .arg_name("my-project")
        .arg_git(template_dir.path())
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(working_dir
        .read("my-project/random.toml")
        .contains(r#"value = "default value""#));
}

#[test]
fn it_accepts_template_values_file_from_environment() {
    let config_dir = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [values]
                my_value = "default value"
                "#},
        )
        .build();

    let template_dir = tempdir()
        .file(
            "my-env-values.toml",
            indoc! {r#"
                [values]
                my_value = "env-file-value"
            "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{my_value}}"
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_dir.path().join("cargo-generate.toml"))
        .arg_name("foobar-project")
        .arg_git(template_dir.path())
        .current_dir(dir.path())
        .env(
            "CARGO_GENERATE_TEMPLATE_VALUES_FILE",
            template_dir.path().join("my-env-values.toml"),
        )
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let random_toml = dbg!(dir.read("foobar-project/random.toml"));
    assert!(random_toml.contains("value = \"env-file-value\""));
}

#[test]
fn it_accepts_bool_in_file() {
    let template = tempdir()
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                v1 = true
                v2 = "true"
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! { r#"
                [placeholders]
                v1 = {type="bool", prompt="?"}

                [conditional.'v1'.placeholders]
                v2 = {type="bool", prompt="?"}

                [conditional.'v2']
                ignore = ["included"]
            "# },
        )
        .file("included", "{{project-name}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg("--silent")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.exists("foobar-project/included").not());
}

#[test]
fn it_accepts_individual_template_values_from_environment() {
    let template = tempdir()
        .file(
            "my-env-values.toml",
            indoc! {r#"
                [values]
                my_value = "env-file-value"
            "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{my_value}}"
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg_git(template.path())
        .current_dir(dir.path())
        .env(
            "CARGO_GENERATE_TEMPLATE_VALUES_FILE",
            template.path().join("my-env-values.toml"),
        )
        .env("CARGO_GENERATE_VALUE_MY_VALUE", "env-def-value")
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let random_toml = dbg!(dir.read("foobar-project/random.toml"));
    assert!(random_toml.contains("value = \"env-def-value\""));
}

#[test]
fn it_accepts_template_values_file_via_flag() {
    let template = tempdir()
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                my_value = "file-value"
            "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{my_value}}"
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg_git(template.path())
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .current_dir(dir.path())
        .env("CARGO_GENERATE_VALUE_MY_VALUE", "env-def-value")
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let random_toml = dbg!(dir.read("foobar-project/random.toml"));
    assert!(random_toml.contains("value = \"file-value\""));
}

#[test]
fn it_accepts_individual_template_values_via_flag() {
    let template = tempdir()
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                my_value = "file-value"
            "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{my_value}}"
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg_git(template.path())
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .arg("--define")
        .arg("my_value=def-value")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let random_toml = dbg!(dir.read("foobar-project/random.toml"));
    assert!(random_toml.contains("value = \"def-value\""));
}

#[test]
fn it_accepts_empty_define_variables() {
    let template = tempdir()
        .file(
            "my-values.toml",
            indoc! {r#"
                [values]
                my_value = "abc"
            "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{my_value}}"
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg_git(template.path())
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .arg("--define")
        .arg("my_value=")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let random_toml = dbg!(dir.read("foobar-project/random.toml"));
    assert!(random_toml.contains("value = \"\""));
}

#[test]
fn it_accepts_values_via_long_option() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg("--define")
        .arg(r#"my_value="content of my-value""#)
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of my-value"));
}

#[test]
fn it_accepts_values_via_short_option() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg("-d")
        .arg(r#"my_value="content of my-value""#)
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of my-value"));
}

#[test]
fn cli_value_overrides_others() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg("--template-values-file")
        .arg(template.path().join("my-values.toml"))
        .arg("-d")
        .arg(r#"my_value="content of cli-value""#)
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("content of cli-value"));
}

#[test]
fn cli_values_are_checked_via_regex() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "{{my_value}}"
                version = "0.1.0"
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [placeholders.my_value]
                type = "string"
                prompt = "What will the name of 'my_value' be?"
                regex = "^$"
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg("-d")
        .arg(r#"my_value="content of my-value""#)
        .arg_path(template.path())
        .current_dir(dir.path())
        .assert()
        .failure();
}
