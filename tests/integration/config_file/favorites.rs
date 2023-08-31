use crate::helpers::prelude::*;

use cargo_generate::Vcs;

fn create_favorite_config(
    name: &str,
    template_path: &Project,
    vcs: Option<Vcs>,
) -> (Project, PathBuf) {
    let project = tempdir()
        .file(
            "cargo-generate",
            format!(
                indoc! {r#"
                    [favorites.{name}]
                    description = "Favorite for the {name} template"
                    git = "{git}"
                    branch = "{branch}"
                    {vcs}
                    "#},
                name = name,
                git = template_path.path().display().to_string().escape_default(),
                branch = "main",
                vcs = if let Some(vcs) = vcs {
                    format!(r#"vcs = "{vcs:?}""#)
                } else {
                    String::from("")
                }
            ),
        )
        .build();
    let path = project.path().join("cargo-generate");
    (project, path)
}

#[test]
fn favorite_with_git_becomes_subfolder() {
    let favorite_template = create_template("favorite-template");
    let git_template = create_template("git-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template, None);
    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_path)
        .arg_name("foobar-project")
        .arg_git(git_template.path())
        .arg("test")
        .current_dir(working_dir.path())
        .assert()
        .failure();
}

#[test]
fn favorite_subfolder_must_be_valid() {
    let template = tempdir()
        .file("Cargo.toml", "")
        .file(
            "inner/Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();
    let working_dir = tempdir().build();

    binary()
        .arg_name("outer")
        .arg(template.path())
        .arg("Cargo.toml")
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("must be a valid folder").from_utf8());

    binary()
        .arg_name("outer")
        .arg(template.path())
        .arg("non-existent")
        .current_dir(working_dir.path())
        .assert()
        .failure(); // Error text is OS specific

    binary()
        .arg_name("outer")
        .arg(template.path())
        .arg(working_dir.path().parent().unwrap())
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid subfolder.").from_utf8());
}

#[test]
fn favorite_with_subfolder() -> anyhow::Result<()> {
    let template = tempdir()
        .file("Cargo.toml", "")
        .file(
            "inner/Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let working_dir = tempdir().build();
    binary()
        .arg_name("outer")
        .arg(template.path())
        .arg("inner")
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(working_dir.read("outer/Cargo.toml").contains("outer"));
    Ok(())
}

#[test]
fn it_can_use_favorites() {
    let favorite_template = create_template("favorite-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template, None);
    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_path)
        .arg_name("favorite-project")
        .arg("test")
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(Repository::open(working_dir.path().join("favorite-project")).is_ok());
    assert!(working_dir
        .read("favorite-project/Cargo.toml")
        .contains(r#"description = "favorite-template""#));
}

#[test]
fn a_favorite_can_set_vcs_to_none_by_default() {
    let favorite_template = create_template("favorite-template");
    let (_config, config_path) =
        create_favorite_config("test", &favorite_template, Some(Vcs::None));
    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_path)
        .arg_name("favorite-project")
        .arg("test")
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(Repository::open(working_dir.path().join("favorite-project")).is_err());
}

#[test]
fn favorites_default_to_git_if_not_defined() {
    let favorite_template = create_template("favorite-template");
    let (_config, config_path) = create_favorite_config("test", &favorite_template, None);
    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_path)
        .arg_name("favorite-project")
        .arg("dummy")
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains(r#"Please check if the Git user / repository exists"#)
                .from_utf8(),
        );
}

#[test]
fn favorites_can_use_default_values() {
    let favorite_template_dir = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
            [package]
            name = "{{project-name}}"
            description = "{{my_value}}"
            version = "0.1.0"
        "#},
        )
        .init_git()
        .build();

    let config_dir = tempdir()
        .file(
            "cargo-generate.toml",
            format!(
                indoc! {r#"
                [favorites.favorite]
                git = "{git}"

                [favorites.favorite.values]
                my_value = "Hello World"
                "#},
                git = favorite_template_dir
                    .path()
                    .display()
                    .to_string()
                    .escape_default(),
            ),
        )
        .build();

    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_dir.path().join("cargo-generate.toml"))
        .arg_name("my-project")
        .arg("favorite")
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(working_dir
        .read("my-project/Cargo.toml")
        .contains(r#"description = "Hello World""#));
}

#[test]
fn favorites_default_value_can_be_overridden_by_environment() {
    let values_dir = tempdir()
        .file(
            "values_file.toml",
            indoc! {r#"
            [values]
            my_value = "Overridden value"
        "#},
        )
        .build();

    let favorite_template_dir = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
            [package]
            name = "{{project-name}}"
            description = "{{my_value}}"
            version = "0.1.0"
        "#},
        )
        .init_git()
        .build();

    let config_dir = tempdir()
        .file(
            "cargo-generate.toml",
            format!(
                indoc! {r#"
                [favorites.favorite]
                git = "{git}"

                [favorites.favorite.values]
                my_value = "Hello World"
                "#},
                git = favorite_template_dir
                    .path()
                    .display()
                    .to_string()
                    .escape_default(),
            ),
        )
        .build();

    let working_dir = tempdir().build();

    binary()
        .arg("--config")
        .arg(config_dir.path().join("cargo-generate.toml"))
        .arg_name("my-project")
        .arg("favorite")
        .current_dir(working_dir.path())
        .env(
            "CARGO_GENERATE_TEMPLATE_VALUES_FILE",
            values_dir.path().join("values_file.toml"),
        )
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(working_dir
        .read("my-project/Cargo.toml")
        .contains(r#"description = "Overridden value""#));
}

#[test]
fn favorite_can_specify_to_be_generated_into_cwd() -> anyhow::Result<()> {
    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
                "#},
        )
        .init_git()
        .build();
    let config_dir = tempdir()
        .file(
            "config.toml",
            format!(
                indoc! {r#"
                [favorites.favorite]
                git = "{git}"
                init = true
                "#},
                git = template.path().display().to_string().escape_default(),
            ),
        )
        .build();

    let dir = tempdir().build();
    binary()
        .arg("--config")
        .arg(config_dir.path().join("config.toml"))
        .arg_name("my-proj")
        .arg("favorite")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.read("Cargo.toml").contains("my-proj"));
    assert!(!dir.path().join(".git").exists());
    Ok(())
}
