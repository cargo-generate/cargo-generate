use crate::helpers::prelude::*;

use cargo_generate::{generate, GenerateArgs, TemplatePath};

#[test]
fn it_allows_generate_call_with_public_args_and_returns_the_generated_path() {
    let cwd_before = std::env::current_dir().unwrap();

    let template = tempdir().init_default_template().init_git().build();

    let dir = tempdir().build().root.path().to_path_buf();

    let args_exposed: GenerateArgs = GenerateArgs {
        template_path: TemplatePath {
            auto_path: None,
            git: Some(format!("{}", template.path().display())),
            branch: Some(String::from("main")),
            tag: None,
            revision: None,
            path: None,
            favorite: None,
            subfolder: None,
            test: false,
        },
        name: Some(String::from("foobar_project")),
        force: true,
        vcs: None,
        verbose: true,
        template_values_file: None,
        silent: false,
        continue_on_error: false,
        quiet: false,
        list_favorites: false,
        config: None,
        bin: true,
        lib: false,
        ssh_identity: None,
        gitconfig: None,
        define: vec![],
        init: false,
        destination: Some(dir.clone()),
        force_git_init: false,
        allow_commands: false,
        overwrite: false,
        other_args: None,
        skip_submodules: false,
    };

    assert_eq!(
        generate(args_exposed).expect("cannot generate project"),
        dir.join("foobar_project")
    );

    assert!(
        std::fs::read_to_string(dir.join("foobar_project").join("Cargo.toml"))
            .expect("cannot read file")
            .contains("foobar_project")
    );

    let cwd_after = std::env::current_dir().unwrap();
    assert!(cwd_after == cwd_before);
}
