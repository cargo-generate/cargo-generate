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
