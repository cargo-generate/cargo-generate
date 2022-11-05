use std::fs;
use std::io::Write;
use std::str;

use crate::helpers::project::Project;
use tempfile::{tempdir, TempDir};

pub struct ProjectBuilder {
    files: Vec<(String, String)>,
    submodules: Vec<(String, String)>,
    root: TempDir,
    git: bool,
    branch: Option<String>,
    tag: Option<String>,
}

pub fn tmp_dir() -> ProjectBuilder {
    ProjectBuilder {
        files: Vec::new(),
        submodules: Vec::new(),
        root: tempdir().unwrap(),
        git: false,
        branch: None,
        tag: None,
    }
}

impl ProjectBuilder {
    /// builds a template with
    /// - one file `Cargo.toml` in it
    /// - one placeholder `project-name`
    pub fn init_default_template(self) -> Self {
        self.file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        )
        .init_git()
    }

    pub fn file(mut self, name: &str, contents: impl AsRef<str>) -> Self {
        self.files
            .push((name.to_string(), contents.as_ref().to_string()));
        self
    }

    pub fn init_git(mut self) -> Self {
        self.git = true;
        self
    }

    pub fn branch(mut self, branch: &str) -> Self {
        self.branch = Some(branch.to_owned());
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = Some(tag.to_owned());
        self
    }

    pub fn add_submodule(
        mut self,
        destination: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        self.submodules.push((destination.into(), path.into()));
        self
    }

    /// On Git >=2.28.0 `init.defaultBranch` can be set to change the default initial branch name
    /// to something other than `master`. Calling this function after the first commit makes sure
    /// the initial branch is named `main` in all our integration tests so that they're not
    /// effected by `init.defaultBranch`.
    fn rename_branch_to_main(&self) {
        use assert_cmd::prelude::*;
        std::process::Command::new("git")
            .arg("branch")
            .arg("--move")
            .arg("main")
            .current_dir(self.root.path())
            .assert()
            .success();
    }

    pub fn build(self) -> Project {
        let path = self.root.path();

        for &(ref file, ref contents) in self.files.iter() {
            let path = path.join(file);
            let parent = path
                .parent()
                .unwrap_or_else(|| panic!("couldn't find parent dir of {:?}", path));

            fs::create_dir_all(parent)
                .unwrap_or_else(|_| panic!("couldn't create {:?} directory", parent));

            fs::File::create(&path)
                .unwrap_or_else(|_| panic!("couldn't create file {:?}", path))
                .write_all(contents.as_ref())
                .unwrap_or_else(|_| panic!("couldn't write to file {:?}: {:?}", path, contents));
        }

        if self.git {
            use assert_cmd::prelude::*;
            use std::process::Command;

            Command::new("git")
                .arg("init")
                .current_dir(path)
                .assert()
                .success();

            if let Some(ref branch) = self.branch {
                // Create dummy content in "main" branch to aid testing

                fs::File::create(path.join("dummy.txt"))
                    .expect("Failed to create dummy")
                    .write_all(b"main dummy")
                    .expect("Couldn't write out dummy text");

                Command::new("git")
                    .arg("add")
                    .arg("dummy.txt")
                    .current_dir(path)
                    .assert()
                    .success();

                Command::new("git")
                    .arg("commit")
                    .arg("--message")
                    .arg("initial main commit")
                    .current_dir(path)
                    .assert()
                    .success();

                self.rename_branch_to_main();

                Command::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg(branch)
                    .current_dir(path)
                    .assert()
                    .success();
            }

            Command::new("git")
                .arg("add")
                .arg("--all")
                .current_dir(path)
                .assert()
                .success();

            self.submodules.iter().for_each(|(d, m)| {
                Command::new("git")
                    .arg("submodule")
                    .arg("add")
                    .arg(m)
                    .arg(d)
                    .current_dir(path)
                    .assert()
                    .success();
            });

            Command::new("git")
                .arg("commit")
                .arg("--message")
                .arg("initial commit")
                .current_dir(path)
                .assert()
                .success();

            if let Some(ref tag) = self.tag {
                Command::new("git")
                    .arg("tag")
                    .arg("-a")
                    .arg(tag)
                    .arg("-m")
                    .arg(format!("our test tag {tag}"))
                    .current_dir(path)
                    .assert()
                    .success();

                for &(ref file, _) in self.files.iter() {
                    let path = path.join(file);
                    fs::remove_file(&path).unwrap_or_else(|_| {
                        panic!("couldn't remove file {path:?}, after committing tag {tag}")
                    });
                }

                Command::new("git")
                    .arg("add")
                    .arg("--all")
                    .current_dir(path)
                    .assert()
                    .success();

                Command::new("git")
                    .arg("commit")
                    .arg("--message")
                    .arg("dummy commit after tag")
                    .current_dir(path)
                    .assert()
                    .success();
            }

            if self.branch.is_some() {
                Command::new("git")
                    .arg("checkout")
                    .arg("main")
                    .current_dir(path)
                    .assert()
                    .success();
            } else {
                self.rename_branch_to_main();
            }
        }

        Project { root: self.root }
    }
}
