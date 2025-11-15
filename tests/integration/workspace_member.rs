use crate::helpers::prelude::*;

#[test]
fn it_should_add_a_new_project_to_the_workspace_members() {
    let workspace_project = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                members = ["c"]
            "#},
        )
        .init_git()
        .build();

    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    binary()
        .arg_name("a")
        .arg_path(template.path())
        .current_dir(workspace_project.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(workspace_project.exists("a/Cargo.toml"));
    assert!(workspace_project
        .read("a/Cargo.toml")
        .contains(r#"name = "a""#));

    // the new project should **not** have an own git repository
    assert!(!workspace_project.exists("a/.git"));

    // pretty printed and also sorted alphabetically
    assert!(workspace_project
        .read("Cargo.toml")
        .contains(indoc! {r#"members = [
            "a",
            "c",
        ]"#}));
}

#[test]
fn it_should_skip_workspace_when_no_workspace_flag_is_set() {
    let workspace_project = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                members = ["c"]
            "#},
        )
        .init_git()
        .build();

    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    binary()
        .arg_name("a")
        .arg_path(template.path())
        .flag_no_workspace()
        .current_dir(workspace_project.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(workspace_project.exists("a/Cargo.toml"));
    assert!(workspace_project
        .read("a/Cargo.toml")
        .contains(r#"name = "a""#));

    // the workspace Cargo.toml should remain unchanged
    let workspace_toml = workspace_project.read("Cargo.toml");
    assert!(workspace_toml.contains(r#"members = ["c"]"#));
    assert!(!workspace_toml.contains(r#""a""#));
}
