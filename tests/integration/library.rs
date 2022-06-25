use crate::helpers::project_builder::tmp_dir;
use cargo_generate::{app_config_path, generate, AppConfig, GenerateArgs, TemplatePath, Vcs};

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

    let args_exposed: GenerateArgs = GenerateArgs {
        template_path: TemplatePath {
            auto_path: None,
            git: Some(format!("{}", template.path().display())),
            branch: Some(String::from("main")),
            path: None,
            favorite: None,
            subfolder: None,
        },
        name: Some(String::from("foobar_project")),
        force: true,
        vcs: Vcs::Git,
        verbose: true,
        template_values_file: None,
        silent: false,
        list_favorites: false,
        config: None,
        bin: true,
        lib: false,
        ssh_identity: None,
        define: vec![],
        init: false,
        destination: None,
        force_git_init: false,
        allow_commands: false,
    };

    let app_config: AppConfig = app_config_path(&args_exposed.config)
        .expect("cannot get app's config path")
        .as_path()
        .try_into()
        .expect("cannot parse app's config path");

    // need to cd to the dir as we aren't running in the cargo shell.
    assert!(std::env::set_current_dir(&dir.root).is_ok());
    assert!(generate(app_config, args_exposed).is_ok());

    assert!(dir
        .read("foobar_project/Cargo.toml")
        .contains("foobar_project"));
}
