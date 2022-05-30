use indoc::indoc;

use assert_cmd::assert::OutputAssertExt;
use predicates::str::PredicateStrExt;

use crate::helpers::{project::binary, project_builder::tmp_dir};

#[test]
fn it_runs_all_hook_types() {
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
            "system-script.rhai",
            indoc! {r#"
                let output = system::command("touch", ["touched_file"]);
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
            post = ["post-script.rhai", "system-script.rhai"]
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
        .arg("--allow-commands")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("script-project/PRE"));
    assert!(dir.exists("script-project/POST"));

    assert!(dir.exists("script-project/touched_file"));

    assert!(dir.read("script-project/PRE").contains("hello"));
    assert!(dir.read("script-project/POST").contains("world"));
}

#[test]
fn it_runs_system_commands() {
    let template = tmp_dir()
        .file(
            "system-script.rhai",
            indoc! {r#"
                let output = system::command("touch", ["touched_file"]);
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            post = ["system-script.rhai"]
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
        .arg("--allow-commands")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("script-project/touched_file"));
}

#[test]
fn it_fails_to_prompt_for_system_commands_in_silent_mode() {
    let template = tmp_dir()
        .file(
            "system-script.rhai",
            indoc! {r#"
                let output = system::command("touch", ["touched_file"]);
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            post = ["system-script.rhai"]
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
        .arg("--silent")
        .current_dir(&dir.path())
        .assert()
        .failure()
        // The error message should instruct the user on how to proceed with silent mode (i.e. by setting the allow flag).
        .stderr(predicates::str::contains("--allow-commands").from_utf8());
}

#[test]
fn it_fails_when_a_system_command_returns_non_zero_exit_code() {
    let template = tmp_dir()
        .file(
            "system-script.rhai",
            indoc! {r#"
                let output = system::command("mkdir", ["invalid_/.dir_name"]);
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            post = ["system-script.rhai"]
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
        .arg("--allow-commands")
        .current_dir(&dir.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains(
                "System command `mkdir invalid_/.dir_name` returned non-zero status",
            )
            .from_utf8(),
        );
}

#[test]
fn it_fails_when_it_cant_execute_system_command() {
    let template = tmp_dir()
        .file(
            "system-script.rhai",
            indoc! {r#"
                let output = system::command("dummy_command_that_doesn't_exist", ["dummy_arg"]);
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            post = ["system-script.rhai"]
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
        .arg("--allow-commands")
        .current_dir(&dir.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains(
                "System command `dummy_command_that_doesn't_exist dummy_arg` failed to execute",
            )
            .from_utf8(),
        );
}
