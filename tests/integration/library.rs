use crate::helpers::project_builder::tmp_dir;
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};

#[test]
fn it_allows_generate_call_with_public_args_and_returns_the_generated_path() {
    let template = tmp_dir().init_default_template().init_git().build();

    let dir = tmp_dir().build().root.into_path();

    let args_exposed: GenerateArgs = GenerateArgs {
        template_path: TemplatePath {
            auto_path: None,
            git: Some(format!("{}", template.path().display())),
            branch: Some(String::from("main")),
            tag: None,
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
        destination: Some(dir.clone()),
        force_git_init: false,
        allow_commands: false,
    };

    assert_eq!(
        generate(args_exposed).expect("cannot generate project"),
        dir.join("foobar_project")
    );

    assert!(
        std::fs::read_to_string(&dir.join("foobar_project").join("Cargo.toml"))
            .expect("cannot read file")
            .contains("foobar_project")
    );
}
