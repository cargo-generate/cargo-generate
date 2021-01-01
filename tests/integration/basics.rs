use predicates;

use crate::helpers::project_builder::dir;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::env;
use std::fs;
use std::process::Command;

fn binary() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn it_substitutes_projectname_in_cargo_toml() {
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

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_substitutes_date() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project Copyright {{ "2018-10-04 18:18:45 +0200" | date: "%Y" }}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("Copyright 2018"));
}

#[test]
fn it_substitutes_os_arch() {
    let template = dir("template")
        .file("some-file", r#"{{os-arch}}"#)
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/some-file")
        .contains(&format!("{}-{}", env::consts::OS, env::consts::ARCH).to_string()));
}

#[test]
fn it_kebabcases_projectname_when_passed_to_flag() {
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

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar_project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_substitutes_cratename_in_a_rust_file() {
    let template = dir("template")
        .file(
            "main.rs",
            r#"
extern crate {{crate_name}};
"#,
        )
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let file = dir.read("foobar-project/main.rs");
    assert!(file.contains("foobar_project"));
    assert!(!file.contains("foobar-project"));
}

#[test]
fn short_commands_work() {
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

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_allows_user_defined_projectname_when_passing_force_flag() {
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

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar_project")
        .arg("--branch")
        .arg("main")
        .arg("--force")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar_project/Cargo.toml")
        .contains("foobar_project"));
}

#[test]
fn it_removes_files_listed_in_genignore() {
    let template = dir("template")
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

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/notme.sh"), true);
    assert_eq!(dir.exists("foobar-project/deleteme.sh"), false);
    assert_eq!(dir.exists("foobar-project/deleteme.trash"), false);
}

#[test]
fn it_prints_ignored_files_with_verbose() {
    let template = dir("template")
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

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .arg("--verbose")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("deleteme.trash").from_utf8());
}

#[test]
fn it_always_removes_genignore_file() {
    let template = dir("template")
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

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/.genignore"), false);
}

#[test]
fn it_always_removes_cargo_ok_file() {
    let template = dir("template")
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

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/.cargo-ok"), false);
}

#[test]
fn it_removes_genignore_files_before_substitution() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file(".cicd_workflow", "i contain a ${{ github }} var")
        .file(".genignore", r#".cicd_workflow"#)
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/.cicd_workflow"), false);
}

#[test]
fn it_does_not_remove_files_from_outside_project_dir() {
    let template = dir("template")
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
            r#"../dangerous.todelete.cargogeneratetests
"#,
        )
        .init_git()
        .build();

    let dir = dir("main").build();

    let dangerous_file = template
        .path()
        .join("..")
        .join("dangerous.todelete.cargogeneratetests");

    fs::write(&dangerous_file, "YOU BETTER NOT").expect(&format!(
        "Could not write {}",
        dangerous_file.to_str().expect("Could not read path.")
    ));

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
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
    let template = dir("template")
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
            r#"../dangerous.todelete.cargogeneratetests
"#,
        )
        .file("./dangerous.todelete.cargogeneratetests", "IM FINE OK")
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(fs::metadata(
        template
            .path()
            .join("dangerous.todelete.cargogeneratetests")
    )
    .expect("should exist")
    .is_file());
}

#[test]
fn it_allows_a_git_branch_to_be_specified() {
    // Build and commit on branch named 'main'
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
        .branch("baz")
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--branch")
        .arg("baz")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_loads_a_submodule() {
    let submodule = dir("submodule")
        .file("README.md", "*JUST A SUBMODULE*")
        .init_git()
        .build();

    let submodule_url = url::Url::from_file_path(submodule.path()).unwrap();
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
        .add_submodule("./submodule/", submodule_url.as_str())
        .build();

    let dir = dir("main").build();
    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
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

    let relative_path = "../".to_string()
        + &template
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

    let dir = dir("main").build();
    binary()
        .arg("generate")
        .arg("--git")
        .arg(relative_path)
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_respects_template_branch_name() {
    let template = dir("template")
        .file("index.html", "My Page")
        .init_git()
        .build();

    Command::new("git")
        .arg("branch")
        .arg("-m")
        .arg("main")
        .arg("gh-pages")
        .current_dir(template.path())
        .assert()
        .success();

    let dir = dir("main").build();
    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("gh-pages")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    Command::new("git")
        .arg("status")
        .current_dir(&dir.path().join("foobar-project"))
        .assert()
        .success()
        .stdout(predicates::str::contains("On branch gh-pages").from_utf8());
}

#[test]
fn it_only_processes_include_files_in_config() {
    let template = dir("template")
        .file(
            "cargo-generate.toml",
            r#"[template]
include = ["included"]
exclude = ["excluded2"]
"#,
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

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

#[test]
fn it_doesnt_process_excluded_files_in_config() {
    let template = dir("template")
        .file(
            "cargo-generate.toml",
            r#"[template]
exclude = ["excluded"]
"#,
        )
        .file("included1", "{{project-name}}")
        .file("included2", "{{project-name}}")
        .file("excluded", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/excluded")
        .contains("{{should-not-process}}"));
    assert!(dir
        .read("foobar-project/included1")
        .contains("foobar-project"));
    assert!(dir
        .read("foobar-project/included2")
        .contains("foobar-project"));
}

#[test]
fn it_warns_on_include_and_exclude_in_config() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file("not-actually-excluded", "{{project-name}}")
        .file(
            "cargo-generate.toml",
            r#"[template]
include = ["Cargo.toml", "not-actually-excluded"]
exclude = ["not-actually-excluded"]
"#,
        )
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("both").from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(dir
        .read("foobar-project/not-actually-excluded")
        .contains("foobar-project"));
}

#[test]
fn it_always_removes_config_file() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file(
            "cargo-generate.toml",
            r#"[template]
"#,
        )
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/cargo-generate.toml"), false);
}

//https://github.com/ashleygwilliams/cargo-generate/issues/181
#[test]
fn it_doesnt_warn_on_config_with_no_ignore() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .file(
            "cargo-generate.toml",
            r#"[template]
"#,
        )
        .init_git()
        .build();
    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("neither").count(0).from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/cargo-generate.toml"), false);
}

#[test]
fn it_doesnt_warn_with_neither_config_nor_ignore() {
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

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed:").count(0).from_utf8())
        .stdout(predicates::str::contains("neither").count(0).from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());
}

#[test]
fn it_applies_filters() {
    let template = dir("template")
        .file(
            "filters.txt",
            r#"kebab-case = {{crate_name | kebab_case}}
    PascalCase = {{crate_name | pascal_case}}
    snake_case = {{crate_name | snake_case}}
    without_suffix = {{crate_name | split: "_" | first}}
    "#,
        )
        .init_git()
        .build();
    let dir = dir("main").build();
    // without_suffix = {{crate_name | split "_project" | first}}

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/filters.txt");
    assert!(cargo_toml.contains("kebab-case = foobar-project"));
    assert!(cargo_toml.contains("PascalCase = FoobarProject"));
    assert!(cargo_toml.contains("snake_case = foobar_project"));
    assert!(cargo_toml.contains("without_suffix = foobar"));
    assert!(!cargo_toml.contains("without_suffix = foobar_project"));
}

#[test]
fn it_processes_dot_github_directory_files() {
    let template = dir("template")
        .file(".github/foo.txt", "{{project-name}}")
        .init_git()
        .build();
    let dir = dir("main").build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
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
    let raw_template = format!("{{% raw %}}{}{{% endraw %}}", raw_body);
    let template = dir("template")
        .file("README.tpl", &raw_template)
        .init_git()
        .build();

    let dir = dir("main").build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.read("foobar-project/README.tpl").contains(raw_body));
}
