use crate::helpers::{project::binary, project_builder::tmp_dir};
use assert_cmd::assert::OutputAssertExt;
use cargo_generate::{generate, Args, Vcs};
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
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("-d")
        .arg("foo=false")
        .current_dir(&dir.path())
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
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("-d")
        .arg("foo=true")
        .current_dir(&dir.path())
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
