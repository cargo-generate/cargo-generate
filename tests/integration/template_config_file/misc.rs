use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use git2::Repository;
use indoc::indoc;

#[test]
fn it_always_removes_config_file() {
    let template = tmp_dir()
        .default_manifest()
        .file(
            "cargo-generate.toml",
            r#"[template]
"#,
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(!dir.exists("foobar-project/cargo-generate.toml"));
}

//https://github.com/ashleygwilliams/cargo-generate/issues/181
#[test]
fn it_doesnt_warn_on_config_with_no_ignore() {
    let template = tmp_dir()
        .default_manifest()
        .file(
            "cargo-generate.toml",
            r#"[template]
"#,
        )
        .init_git()
        .build();
    let dir = tmp_dir().build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("neither").count(0).from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(!dir.exists("foobar-project/cargo-generate.toml"));
}

#[test]
fn a_template_can_specify_to_be_generated_into_cwd() -> anyhow::Result<()> {
    let template = tmp_dir()
        .default_manifest()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                init = true
                "#},
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("Cargo.toml"));
    assert!(!dir.path().join(".git").exists());
    Ok(())
}

#[test]
fn vsc_none_can_be_specified_in_the_template() {
    // Build and commit on branch named 'main'
    let template = tmp_dir()
        .default_manifest()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                vcs = "None"
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
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(Repository::open(dir.path().join("foobar-project")).is_err());
}
