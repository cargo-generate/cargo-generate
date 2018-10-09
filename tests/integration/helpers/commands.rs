extern crate predicates;

use helpers::project::Project;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

pub struct CargoGenerateCommandBuilder<'a> {
    branch: Option<&'a str>,
    dir: &'a Project,
    force: bool,
    git_path: &'a Project,
    project_name: &'a str,
}

impl<'a> CargoGenerateCommandBuilder<'a> {

    pub fn new(dir: &'a Project, project_name: &'a str, git_path: &'a Project) -> CargoGenerateCommandBuilder<'a> {
        CargoGenerateCommandBuilder {
            branch: None,
            dir: dir,
            force: false,
            git_path: git_path,
            project_name: project_name,
        }
    }

    pub fn branch(mut self, branch: &'a str) -> CargoGenerateCommandBuilder<'a> {
        self.branch = Some(branch);
        self
    }

    pub fn force(mut self) -> CargoGenerateCommandBuilder<'a> {
        self.force = true;
        self
    }

    pub fn build(&self) {
        let mut cmd = Command::main_binary().unwrap();
        cmd.arg("generate");
        // It the use specified a branch?
        if let Some(ref branch) = self.branch {
            cmd.arg("--branch")
               .arg(branch);
        }
        cmd.arg("--git")
           .arg(self.git_path.path());
        cmd.arg("--name")
           .arg(self.project_name);
        if self.force {
            cmd.arg("--force");
        }
        cmd.current_dir(self.dir.path())
           .assert()
           .success()
           .stdout(predicates::str::contains("Done!").from_utf8());
    }

}

pub fn generate_project(dir: &Project, project_name: &str, template: &Project) {
    CargoGenerateCommandBuilder::new(dir, project_name, template).build()
}

pub fn force_generate_project(dir: &Project, project_name: &str, template: &Project) {
    CargoGenerateCommandBuilder::new(dir, project_name, template)
        .force()
        .build()
}

pub fn generate_project_with_branch(
    dir: &Project,
    project_name: &str,
    template: &Project,
    branch: &str,
) {
    CargoGenerateCommandBuilder::new(dir, project_name, template)
        .branch(branch)
        .build()
}
