use crate::helpers::project_builder::tmp_dir;
use cargo_generate::{generate, Args, Vcs};

#[test]
fn it_allows_generate_call_with_public_args() {
    let template = tmp_dir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    let args_exposed: Args = Args {
        git: Some(format!("{}", template.path().display())),
        path: None,
        branch: Some(String::from("main")),
        subfolder: None,
        name: Some(String::from("foobar_project")),
        force: true,
        vcs: Vcs::Git,
        verbose: true,
        template_values_file: None,
        silent: false,
        list_favorites: false,
        config: None,
        favorite: None,
        bin: true,
        lib: false,
        ssh_identity: None,
        define: vec![],
        init: false,
    };
    // need to cd to the dir as we aren't running in the cargo shell.
    assert!(std::env::set_current_dir(&dir.root).is_ok());
    assert!(generate(args_exposed).is_ok());

    assert!(dir
        .read("foobar_project/Cargo.toml")
        .contains("foobar_project"));
}

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

[conditional.'foo']
include = ["included"]
"#,
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    let args_exposed: Args = Args {
        git: Some(format!("{}", template.path().display())),
        path: None,
        branch: None,
        subfolder: None,
        name: Some(String::from("foobar-project")),
        force: false,
        vcs: Vcs::Git,
        verbose: false,
        template_values_file: None,
        silent: true,
        list_favorites: false,
        config: None,
        favorite: None,
        bin: false,
        lib: false,
        ssh_identity: None,
        define: vec!["foo=false".into()],
        init: false,
    };

    // need to cd to the dir as we aren't running in the cargo shell.
    assert!(std::env::set_current_dir(&dir.root).is_ok());
    assert!(generate(args_exposed).is_ok());

    assert!(!dir.exists("foobar-project/included"));
}

#[test]
fn it_can_conditionally_include_files2() {
    let template = tmp_dir()
        .file(
            "cargo-generate.toml",
            r#"
[template]
exclude = ["excluded2"]

[placeholders]
foo = {type="bool", prompt="?"}

[conditional.'!foo']
ignore = ["included"]

[conditional.'foo']
include = ["included"]
"#,
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tmp_dir().build();

    let args_exposed: Args = Args {
        git: Some(format!("{}", template.path().display())),
        path: None,
        branch: None,
        subfolder: None,
        name: Some(String::from("foobar-project")),
        force: false,
        vcs: Vcs::Git,
        verbose: false,
        template_values_file: None,
        silent: true,
        list_favorites: false,
        config: None,
        favorite: None,
        bin: false,
        lib: false,
        ssh_identity: None,
        define: vec!["foo=true".into()],
        init: false,
    };

    // need to cd to the dir as we aren't running in the cargo shell.
    assert!(std::env::set_current_dir(&dir.root).is_ok());
    assert!(generate(args_exposed).is_ok());

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
