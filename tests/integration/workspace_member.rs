use crate::helpers::prelude::*;

#[test]
fn it_should_not_rewrite_manifest_when_new_member_is_already_covered_by_glob() {
    // A `crates/*` glob already implicitly covers a new `crates/a` crate, so
    // cargo-generate should leave the workspace manifest untouched — matching
    // `cargo new`'s behavior. See `update_manifest_with_new_member` in cargo.
    let workspace_project = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                members = ["crates/*"]
            "#},
        )
        // ensure `crates/` exists so we can `cd` into it below
        .file("crates/.gitkeep", "")
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
        .current_dir(workspace_project.path().join("crates"))
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(workspace_project.exists("crates/a/Cargo.toml"));

    // manifest should be unchanged (still just the glob, no `crates/a` literal).
    let workspace_toml = workspace_project.read("Cargo.toml");
    assert!(workspace_toml.contains(r#"members = ["crates/*"]"#));
    assert!(!workspace_toml.contains(r#""crates/a""#));
}

#[test]
fn it_should_skip_workspace_when_target_matches_workspace_exclude() {
    let workspace_project = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                members = ["crates/c1"]
                exclude = ["crates/a"]
            "#},
        )
        .file(
            "crates/c1/Cargo.toml",
            indoc! {r#"
                [package]
                name = "c1"
                version = "0.1.0"
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
        .current_dir(workspace_project.path().join("crates"))
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    // the new project was created…
    assert!(workspace_project.exists("crates/a/Cargo.toml"));

    // …but the workspace manifest was left untouched because the target
    // path is listed in `workspace.exclude`.
    let workspace_toml = workspace_project.read("Cargo.toml");
    assert!(workspace_toml.contains(r#"members = ["crates/c1"]"#));
    assert!(workspace_toml.contains(r#"exclude = ["crates/a"]"#));
    assert!(!workspace_toml.contains(r#""crates/a","#));
}

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

/// Regression test for https://github.com/cargo-generate/cargo-generate/issues/1648:
/// when generating from a nested directory (e.g. `crates/`) inside a workspace,
/// the newly generated crate should still be added to the workspace's members list
/// at the workspace root — mirroring what `cargo new` does by walking up the
/// directory tree until a workspace manifest is found. The member entry must be
/// the path relative to the workspace root, joined with forward slashes.
#[test]
fn it_should_add_a_new_project_from_a_nested_directory_to_the_workspace_members() {
    let workspace_project = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                members = ["crates/c1"]
            "#},
        )
        .file(
            "crates/c1/Cargo.toml",
            indoc! {r#"
                [package]
                name = "c1"
                version = "0.1.0"
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
        .current_dir(workspace_project.path().join("crates"))
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(workspace_project.exists("crates/a/Cargo.toml"));
    assert!(workspace_project
        .read("crates/a/Cargo.toml")
        .contains(r#"name = "a""#));

    // the new project should **not** have an own git repository
    assert!(!workspace_project.exists("crates/a/.git"));

    // the workspace root Cargo.toml should have picked up the new nested member,
    // relative to the workspace root, and remain sorted alphabetically
    assert!(workspace_project
        .read("Cargo.toml")
        .contains(indoc! {r#"members = [
            "crates/a",
            "crates/c1",
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
