use crate::helpers::prelude::*;

use cargo_generate::{generate, GenerateArgs, TemplatePath};

/// Spell out every field of `GenerateArgs` and `TemplatePath`.
///
/// Compiled in **both** feature configurations on purpose: this is the
/// regression test proving the CLI-shaped API keeps its exact shape when
/// the `git` cargo feature is off. If a field ever picks up a `#[cfg]`,
/// this stops compiling.
fn public_args(git_url: String, destination: PathBuf) -> GenerateArgs {
    GenerateArgs {
        template_path: TemplatePath {
            auto_path: None,
            git: Some(git_url),
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
        destination: Some(destination),
        force_git_init: false,
        allow_commands: false,
        overwrite: false,
        other_args: None,
        skip_submodules: false,
        no_workspace: false,
    }
}

#[cfg(feature = "git")]
#[test]
fn it_allows_generate_call_with_public_args_and_returns_the_generated_path() {
    let cwd_before = std::env::current_dir().unwrap();

    let template = tempdir().init_default_template().init_git().build();

    let dir = tempdir().build().root.keep();

    let args_exposed = public_args(format!("{}", template.path().display()), dir.clone());

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

/// Same call, same argument shape, feature off: it fails with the feature
/// notice rather than failing to compile, and writes nothing.
#[cfg(not(feature = "git"))]
#[test]
fn generate_with_a_git_template_bails_without_the_git_feature() {
    let dir = tempdir().build().root.keep();

    let args_exposed = public_args(
        String::from("https://github.com/cargo-generate/cargo-generate.git"),
        dir.clone(),
    );

    let err = generate(args_exposed)
        .expect_err("a git template must not succeed without the `git` feature")
        .to_string();

    assert!(err.contains("`git` cargo feature"), "{err}");
    assert!(dir.join("foobar_project").exists().not());
}
