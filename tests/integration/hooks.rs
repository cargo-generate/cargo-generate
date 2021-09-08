use indoc::indoc;

use assert_cmd::assert::OutputAssertExt;
use predicates::str::PredicateStrExt;

use crate::helpers::{project::binary, project_builder::tmp_dir};

#[test]
fn it_runs_scripts() {
    let template = tmp_dir()
        .file(
            "pre-script.rhai",
            indoc! {r#"
            file::rename("PRE-TEST", "PRE");
        "#},
        )
        .file(
            "post-script.rhai",
            indoc! {r#"
            file::rename("POST-TEST", "POST");
        "#},
        )
        .file(
            "PRE-TEST",
            indoc! {r#"
            {{pre}};
        "#},
        )
        .file(
            "POST-TEST",
            indoc! {r#"
            {{post}};
        "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [template]
            exclude = ["PRE-TEST", "POST"]

            [hooks]
            pre = ["pre-script.rhai"]
            post = ["post-script.rhai"]
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
        .arg("script-project")
        .arg("-d")
        .arg("pre=hello")
        .arg("-d")
        .arg("post=world")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("script-project/PRE"));
    assert!(dir.exists("script-project/POST"));

    assert!(dir.read("script-project/PRE").contains("hello"));
    assert!(dir.read("script-project/POST").contains("world"));
}
