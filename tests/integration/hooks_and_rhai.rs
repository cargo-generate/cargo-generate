use crate::helpers::prelude::*;

#[test]
fn it_runs_all_hook_types() {
    let template = tempdir()
        .file(
            "init-script.rhai",
            indoc! {r#"
            print("init-script has run");
        "#},
        )
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
            init = ["init-script.rhai"]
            pre = ["pre-script.rhai"]
            post = ["post-script.rhai", "system-script.rhai"]
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .arg("-d")
        .arg("pre=hello")
        .arg("-d")
        .arg("post=world")
        .arg("--allow-commands")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("init-script has run").from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("script-project/PRE"));
    assert!(dir.exists("script-project/POST"));

    assert!(dir.exists("script-project/touched_file"));

    assert!(dir.read("script-project/PRE").contains("hello"));
    assert!(dir.read("script-project/POST").contains("world"));
}

#[test]
fn it_runs_system_commands() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .arg("--allow-commands")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("script-project/touched_file"));
}

#[test]
fn it_fails_to_prompt_for_system_commands_in_silent_mode() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .arg("--silent")
        .current_dir(dir.path())
        .assert()
        .failure()
        // The error message should instruct the user on how to proceed with silent mode (i.e. by setting the allow flag).
        .stderr(predicates::str::contains("--allow-commands").from_utf8());
}

#[test]
fn it_fails_when_a_system_command_returns_non_zero_exit_code() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .arg("--allow-commands")
        .current_dir(dir.path())
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
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .arg("--allow-commands")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains(
                "System command `dummy_command_that_doesn't_exist dummy_arg` failed to execute",
            )
            .from_utf8(),
        );
}

#[test]
fn it_can_change_case() {
    let template = tempdir()
        .file(
            "pre-script.rhai",
            indoc! {r#"
            print(to_kebab_case("kebab case"));
            print(to_lower_camel_case("lower camel case"));
            print(to_pascal_case("pascal case"));
            print(to_shouty_kebab_case("shouty kebab case"));
            print(to_shouty_snake_case("shouty snake case"));
            print(to_snake_case("snake case"));
            print(to_title_case("title case"));
            print(to_upper_camel_case("upper camel case"));
        "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            pre = ["pre-script.rhai"]
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("kebab-case"))
        .stdout(predicates::str::contains("lowerCamelCase"))
        .stdout(predicates::str::contains("PascalCase"))
        .stdout(predicates::str::contains("SHOUTY-KEBAB-CASE"))
        .stdout(predicates::str::contains("SHOUTY_SNAKE_CASE"))
        .stdout(predicates::str::contains("snake_case"))
        .stdout(predicates::str::contains("Title Case"))
        .stdout(predicates::str::contains("UpperCamelCase"));
}

#[test]
fn can_change_variables_from_pre_hook() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            pre = ["pre-script.rhai"]
            "#},
        )
        .file(
            "pre-script.rhai",
            indoc! {r#"
                variable::set("foo", "bar");
            "#},
        )
        .file(
            "PRE-TEST",
            indoc! {r#"
                {{foo}};
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("script-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("script-project/PRE-TEST"));
    assert!(dir.read("script-project/PRE-TEST").contains("bar"));
}

#[test]
fn init_hook_can_set_project_name() {
    let template = tempdir()
        .file(
            "init.rhai",
            indoc! {r#"
                variable::set("project-name", "ProjectBar");
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            init = ["init.rhai"]
            "#},
        )
        .file(
            "generated.txt",
            indoc! {r#"
            {{crate_name}}
        "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("project-bar/generated.txt"));
    assert!(dir
        .read("project-bar/generated.txt")
        .contains("project_bar"));
}

#[test]
fn init_hook_can_change_project_name_but_keeps_cli_name_for_destination() {
    let template = tempdir()
        .file(
            "init.rhai",
            indoc! {r#"
                variable::set("project-name", "bar");
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            init = ["init.rhai"]
            "#},
        )
        .file(
            "generated.txt",
            indoc! {r#"
            {{crate_name}}
        "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foo")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foo/generated.txt"));
    assert!(dir.read("foo/generated.txt").contains("bar"));
}

#[test]
fn init_hook_can_change_project_name_but_keeps_init_destination() {
    let template = tempdir()
        .file(
            "init.rhai",
            indoc! {r#"
                variable::set("project-name", "bar");
            "#},
        )
        .file(
            "cargo-generate.toml",
            indoc! {r#"
            [hooks]
            init = ["init.rhai"]
            "#},
        )
        .file(
            "generated.txt",
            indoc! {r#"
            {{crate_name}}
        "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foo")
        .arg("--init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("generated.txt"));
    assert!(dir.read("generated.txt").contains("bar"));
}

#[test]
fn rhai_filter_invokes_rhai_script() {
    let template = tempdir()
        .file(
            "filter-script.rhai",
            indoc! {r#"
                "content from RHAI"
            "#},
        )
        .file(
            "file_to_expand.txt",
            indoc! {r#"
                {{"filter-script.rhai"|rhai}}
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("filter-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done"));

    assert!(dir
        .read("filter-project/file_to_expand.txt")
        .contains("content from RHAI"));
}

#[test]
fn missing_rhai_filter_fails_prints_warnings() {
    let template = tempdir()
        .file(
            "file_to_expand.txt",
            indoc! {r#"
                {{"filter-script.rhai"|rhai}}
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("filter-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Filter script filter-script.rhai not found",
        ));

    assert!(dir
        .read("filter-project/file_to_expand.txt")
        .contains(r#"{{"filter-script.rhai"|rhai}}"#));
}
