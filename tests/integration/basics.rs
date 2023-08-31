use crate::helpers::prelude::*;

#[test]
fn it_can_use_a_plain_folder() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!")
                .and(predicates::str::contains(format!(
                    "Favorite `{}` not found in config, using it as a local path",
                    template.path().display()
                )))
                .from_utf8(),
        );

    let repo = git2::Repository::open(dir.path().join("foobar-project")).unwrap();
    let references = repo.references().unwrap().count();
    assert_eq!(0, references);
}

#[test]
fn it_can_use_a_specified_path() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg("--path")
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let repo = git2::Repository::open(dir.path().join("foobar-project")).unwrap();
    let references = repo.references().unwrap().count();
    assert_eq!(0, references);
}

#[test]
fn it_substitutes_projectname_in_cargo_toml() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_substitutes_authors_and_username() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
authors = "{{authors}}"
description = "A wonderful project by {{username}}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .env("CARGO_EMAIL", "Email")
        .env("CARGO_NAME", "Author")
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains(r#"authors = "Author <Email>""#));
    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains(r#"description = "A wonderful project by Author""#));
}

#[test]
fn it_substitutes_os_arch() {
    let template = tempdir()
        .file("some-file", r#"{{os-arch}}"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.read("foobar-project/some-file").contains(&format!(
        "{}-{}",
        env::consts::OS,
        env::consts::ARCH
    )));
}

#[test]
fn it_kebabcases_projectname_when_passed_to_flag() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar_project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_substitutes_cratename_in_a_rust_file() {
    let template = tempdir()
        .file(
            "main.rs",
            r#"
extern crate {{crate_name}};
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let file = dir.read("foobar-project/main.rs");
    assert!(file.contains("foobar_project"));
    assert!(!file.contains("foobar-project"));
}

#[test]
fn short_commands_work() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_can_generate_inside_existing_repository() -> anyhow::Result<()> {
    let template = tempdir()
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
    let dir = tempdir().build();
    binary()
        .arg_git(template.path())
        .arg_name("outer")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("outer/Cargo.toml").contains("outer"));
    let outer_project_dir = dir.path().join("outer");
    let outer_repo = git2::Repository::discover(&outer_project_dir)?;

    binary()
        .arg_git(template.path())
        .arg_name("inner")
        .current_dir(&outer_project_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("outer/inner/Cargo.toml").contains("inner"));
    let inner_project_dir = outer_project_dir.join("inner");
    let inner_repo = git2::Repository::discover(inner_project_dir)?;
    assert_eq!(outer_repo.path(), inner_repo.path());
    Ok(())
}

#[test]
fn it_can_generate_into_cwd() -> anyhow::Result<()> {
    let template = tempdir()
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
    let dir = tempdir().build();
    binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .flag_init()
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("Cargo.toml").contains("my-proj"));
    assert!(!dir.path().join(".git").exists());
    Ok(())
}

#[test]
fn it_can_generate_into_existing_git_dir() -> anyhow::Result<()> {
    let template = tempdir()
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
    let dir = tempdir().file(".git/config", "foobar").build();
    binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .flag_init()
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("Cargo.toml").contains("my-proj"));
    assert!(dir.read(".git/config").contains("foobar"));
    Ok(())
}

#[test]
fn it_can_generate_at_given_path() -> anyhow::Result<()> {
    let template = tempdir()
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
    let dir = tempdir().build();
    let dest = dir.path().join("destination");
    fs::create_dir(&dest).expect("can create directory");
    binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .arg("--destination")
        .arg(&dest)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir
        .read("destination/my-proj/Cargo.toml")
        .contains("my-proj"));
    Ok(())
}

#[test]
fn it_refuses_to_overwrite_files() -> anyhow::Result<()> {
    let template = tempdir()
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
    let dir = tempdir().build();
    let _ = binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .flag_init()
        .current_dir(dir.path())
        .status();
    binary()
        .arg_git(template.path())
        .arg_name("overwritten-proj")
        .flag_init()
        .current_dir(dir.path())
        .assert()
        .failure();
    assert!(dir.read("Cargo.toml").contains("my-proj"));
    Ok(())
}

#[test]
fn it_can_overwrite_files() -> anyhow::Result<()> {
    let template = tempdir()
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
    let dir = tempdir().build();
    let _ = binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .flag_init()
        .current_dir(dir.path())
        .status();
    binary()
        .arg_git(template.path())
        .arg_name("overwritten-proj")
        .flag_init()
        .arg("--overwrite")
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(dir.read("Cargo.toml").contains("overwritten-proj"));
    Ok(())
}

#[test]
fn it_allows_user_defined_projectname_when_passing_force_flag() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar_project")
        .arg_branch("main")
        .arg("--force")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar_project/Cargo.toml")
        .contains("foobar_project"));
}

#[test]
fn it_removes_files_listed_in_genignore() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file(
            ".genignore",
            r#"deleteme.sh
*.trash
"#,
        )
        .file("deleteme.sh", r#"Nothing to see here"#)
        .file("deleteme.trash", r#"This is trash"#)
        .file("notme.sh", r#"I'm here!"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/notme.sh"));
    assert!(dir.exists("foobar-project/deleteme.sh").not());
    assert!(dir.exists("foobar-project/deleteme.trash").not());
}

#[test]
fn it_prints_ignored_files_with_verbose() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file(
            ".genignore",
            r#"deleteme.sh
*.trash
"#,
        )
        .file("deleteme.trash", r#"This is trash"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg("--verbose")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("deleteme.trash").from_utf8());
}

#[test]
fn it_always_removes_genignore_file() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file(".genignore", r#"farts"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/.genignore").not());
}

#[test]
fn it_always_removes_cargo_ok_file() {
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
        .file(".genignore", r#"farts"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/.cargo-ok").not());
}

#[test]
fn it_removes_genignore_files_before_substitution() {
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
        .file(".cicd_workflow", "i contain a ${{ github }} var")
        .file(".genignore", r#".cicd_workflow"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/.cicd_workflow").not());
}

#[test]
fn it_does_not_remove_files_from_outside_project_dir() {
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
        .file(
            ".genignore",
            r#"../dangerous.todelete.cargogeneratetests
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    let dangerous_file = template
        .path()
        .join("..")
        .join("dangerous.todelete.cargogeneratetests");

    fs::write(&dangerous_file, "YOU BETTER NOT").unwrap_or_else(|_| {
        panic!(
            "Could not write {}",
            dangerous_file.to_str().expect("Could not read path.")
        )
    });

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(fs::metadata(&dangerous_file)
        .expect("should exist")
        .is_file());
    fs::remove_file(&dangerous_file).expect("failed to clean up test file");
}

#[test]
fn errant_ignore_entry_doesnt_affect_template_files() {
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
        .file(
            ".genignore",
            r#"../dangerous.todelete.cargogeneratetests
"#,
        )
        .file("./dangerous.todelete.cargogeneratetests", "IM FINE OK")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(fs::metadata(
        template
            .path()
            .join("dangerous.todelete.cargogeneratetests")
    )
    .unwrap()
    .is_file());
}

#[test]
fn it_loads_a_submodule() {
    let submodule = tempdir()
        .file("README.md", "*JUST A SUBMODULE*")
        .init_git()
        .build();

    let submodule_url = url::Url::from_file_path(submodule.path()).unwrap();
    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! { r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .add_submodule("./submodule/", submodule_url.as_str())
        .build();

    let dir = tempdir().build();
    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(dir
        .read("foobar-project/submodule/README.md")
        .contains("*JUST A SUBMODULE*"));
}

#[test]
fn it_allows_relative_paths() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! { r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let relative_path = {
        let mut relative_path = std::path::PathBuf::new();
        relative_path.push("../");
        relative_path.push(template.path().file_name().unwrap().to_str().unwrap());
        relative_path
    };

    let dir = tempdir().build();
    binary()
        .arg_git(relative_path)
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_respects_template_branch_name() {
    let template = tempdir().file("index.html", "My Page").init_git().build();

    Command::new("git")
        .arg("branch")
        .arg("-m")
        .arg("main")
        .arg("gh-pages")
        .current_dir(template.path())
        .assert()
        .success();

    let dir = tempdir().build();
    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("gh-pages")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    Command::new("git")
        .arg("status")
        .current_dir(dir.path().join("foobar-project"))
        .assert()
        .success()
        .stdout(predicates::str::contains("On branch gh-pages").from_utf8());
}

#[test]
fn it_doesnt_warn_with_neither_config_nor_ignore() {
    let template = tempdir()
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
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed:").count(0).from_utf8())
        .stdout(predicates::str::contains("neither").count(0).from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());
}

#[test]
fn it_processes_dot_github_directory_files() {
    let template = tempdir()
        .file(".github/foo.txt", "{{project-name}}")
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.read("foobar-project/.github/foo.txt"), "foobar-project");
}

#[test]
fn it_ignore_tags_inside_raw_block() {
    let raw_body = r#"{{badges}}
# {{crate}} {{project-name}}
{{readme}}
{{license}}
## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
This project try follow rules:
* [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
* [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
_This README was generated with [cargo-readme](https://github.com/livioribeiro/cargo-readme) from [template](https://github.com/xoac/crates-io-lib-template)
"#;
    let raw_template = format!("{{% raw %}}{raw_body}{{% endraw %}}");
    let template = tempdir()
        .file("README.tpl", raw_template)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let template = dir.read("foobar-project/README.tpl");
    assert!(template.contains("{{badges}}"));
    assert!(template.contains("{{crate}}"));
    assert!(template.contains("{{project-name}}"));
    assert!(template.contains("{{readme}}"));
    assert!(template.contains("{{license}}"));
}

#[test]
fn it_uses_vsc_none_to_avoid_initializing_repository() {
    // Build and commit on branch named 'main'
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("--vcs")
        .arg("nONE")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(Repository::open(dir.path().join("foobar-project")).is_err());
}

#[test]
fn it_provides_crate_type_lib() {
    // Build and commit on branch named 'main'
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "this is a {{crate_type}}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("--lib")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("this is a lib"));
}

#[test]
fn it_provides_crate_type_bin() {
    // Build and commit on branch named 'main'
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "this is a {{crate_type}}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("this is a bin"));
}

#[test]
fn it_skips_substitution_for_random_garbage_in_cargo_toml() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{function fart() { return "pfffttt"; } fart();}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.read("foobar-project/Cargo.toml").contains("fart"));
}

#[test]
fn it_skips_substitution_for_unknown_variables_in_cargo_toml() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{ project-name }}"
description = "{{ project-description }}"
description2 = "{{ project-some-other-thing }}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project"),
        "project-name was not substituted"
    );
    assert!(!dir
        .read("foobar-project/Cargo.toml")
        .contains("{{ project-description }}"));
    assert!(!dir
        .read("foobar-project/Cargo.toml")
        .contains("{{ project-some-other-thing }}"));
}

#[test]
fn error_message_for_invalid_repo_or_user() {
    let dir = tempdir().build();

    binary()
        .arg_git("sassman/cli-template-rs-xx")
        .arg_name("favorite-project")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains(r#"Error: Please check if the Git user / repository exists"#)
                .from_utf8(),
        );
}
