extern crate predicates;

use helpers::project::Project;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

pub fn generate_project(
    dir: &Project,
    project_name: &str,
    template: &Project,
) {
    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg(project_name)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}

pub fn force_generate_project(
    dir: &Project,
    project_name: &str,
    template: &Project,
) {
    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg(project_name)
        .arg("--force")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}

pub fn generate_project_with_branch(
    dir: &Project,
    project_name: &str,
    template: &Project,
    branch: &str
) {
    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--branch")
        .arg(branch)
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg(project_name)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}
