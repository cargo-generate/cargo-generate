use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use indoc::indoc;

#[test]
fn it_only_processes_include_files_in_config() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                include = ["included"]
                exclude = ["excluded2"]
            "#},
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
        .arg("--branch")
        .arg("main")
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
fn it_doesnt_process_excluded_files_in_config() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                exclude = ["excluded"]
            "#},
        )
        .file("included1", "{{project-name}}")
        .file("included2", "{{project-name}}")
        .file("excluded", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/excluded")
        .contains("{{should-not-process}}"));
    assert!(dir
        .read("foobar-project/included1")
        .contains("foobar-project"));
    assert!(dir
        .read("foobar-project/included2")
        .contains("foobar-project"));
}

#[test]
fn it_warns_on_include_and_exclude_in_config() {
    let template = tmp_dir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .file("not-actually-excluded", "{{project-name}}")
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                include = ["Cargo.toml", "not-actually-excluded"]
                exclude = ["not-actually-excluded"]
            "#},
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("both").from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(dir
        .read("foobar-project/not-actually-excluded")
        .contains("foobar-project"));
}
