use std::ops::Not;

use crate::helpers::{project::binary, project_builder::tmp_dir};
use assert_cmd::assert::OutputAssertExt;
use predicates::str::PredicateStrExt;

#[test]
fn it_can_conditionally_include_files() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            r#"
[template]
exclude = ["excluded1", "excluded2"]

[placeholders]
foo = {type="bool", prompt="?"}

[conditional.'!foo']
ignore = ["included"]
"#,
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("foo=false")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(!dir.exists("foobar-project/included"));
}

#[test]
fn it_can_conditionally_include_files2() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            r#"
[template]
exclude = ["excluded1", "excluded2"]

[placeholders]
foo = {type="bool", prompt="?"}

[conditional.'!foo']
ignore = ["included"]
"#,
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("foo=true")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/included")
        .contains("foobar-project"));

    assert!(dir
        .read("foobar-project/excluded1")
        .contains("{{should-not-process}}"));
    assert!(dir
        .read("foobar-project/excluded2")
        .contains("{{should-not-process}}"));
}

#[test]
fn it_can_ask_placeholders_in_multiple_levels() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            r#"
[placeholders]
v1 = {type="bool", prompt="?"}

[conditional.'v1'.placeholders]
v2 = {type="bool", prompt="?"}

[conditional.'v2']
ignore = ["included"]
"#,
        )
        .file("included", "{{project-name}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("--silent")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("v1=true")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error:").from_utf8());
}

#[test]
fn it_supports_conditions_in_multiple_levels() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            r#"
[placeholders]
v1 = {type="bool", prompt="?"}

[conditional.'v1'.placeholders]
v2 = {type="bool", prompt="?"}

[conditional.'v2']
ignore = ["included"]
"#,
        )
        .file("included", "{{project-name}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("--silent")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("v1=true")
        .arg("-d")
        .arg("v2=true")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.exists("foobar-project/included").not());
}
