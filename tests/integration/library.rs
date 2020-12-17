use crate::helpers::project_builder::dir;
use cargo_generate::{generate, Args};

#[test]
fn it_allows_generate_call_with_public_args() {
    let template = dir("template")
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

    let dir = dir("main").build();

    let args_exposed: Args = Args {
        git: format!("{}", template.path().display()),
        branch: Some(String::from("main")),
        name: Some(String::from("foobar_project")),
        force: true,
        verbose: true,
    };
    // need to cd to the dir as we aren't running in the cargo shell.
    assert!(std::env::set_current_dir(&dir.root).is_ok());
    assert!(generate(args_exposed).is_ok());

    assert!(dir
        .read("foobar_project/Cargo.toml")
        .contains("foobar_project"));
}
