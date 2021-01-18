use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str;
use std::sync::atomic::*;

use crate::helpers::project::Project;
use remove_dir_all::remove_dir_all;

static CNT: AtomicUsize = AtomicUsize::new(0);
thread_local!(static IDX: usize = CNT.fetch_add(1, Ordering::SeqCst));

pub struct ProjectBuilder {
    files: Vec<(String, String)>,
    submodules: Vec<(String, String)>,
    root: PathBuf,
    git: bool,
    branch: Option<String>,
}

pub fn dir(name: &str) -> ProjectBuilder {
    ProjectBuilder {
        files: Vec::new(),
        submodules: Vec::new(),
        root: root(name),
        git: false,
        branch: None,
    }
}

fn root(name: &str) -> PathBuf {
    let idx = IDX.with(|x| *x);

    let mut me = env::current_exe().expect("couldn't find current exe");
    me.pop(); // chop off exe name
    me.pop(); // chop off `deps`
    me.pop(); // chop off `debug` / `release`
    me.push("generated-tests");
    me.push(&format!("test-{}-{}", idx, name));

    me
}

impl ProjectBuilder {
    pub fn file(mut self, name: &str, contents: &str) -> ProjectBuilder {
        self.files.push((name.to_string(), contents.to_string()));
        self
    }

    pub fn init_git(mut self) -> ProjectBuilder {
        self.git = true;
        self
    }

    pub fn branch(mut self, branch: &str) -> ProjectBuilder {
        self.branch = Some(branch.to_owned());
        self
    }

    pub fn add_submodule<I: Into<String>>(mut self, destination: I, path: I) -> ProjectBuilder {
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
            .current_dir(&self.root)
            .assert()
            .success();
    }

    pub fn build(self) -> Project {
        drop(remove_dir_all(&self.root));
        fs::create_dir_all(&self.root)
            .unwrap_or_else(|_| panic!("couldn't create {:?} directory", self.root));

        for &(ref file, ref contents) in self.files.iter() {
            let dst = self.root.join(file);
            let parent = dst
                .parent()
                .unwrap_or_else(|| panic!("couldn't find parent dir of {:?}", dst));

            fs::create_dir_all(parent)
                .unwrap_or_else(|_| panic!("couldn't create {:?} directory", parent));

            fs::File::create(&dst)
                .unwrap_or_else(|_| panic!("couldn't create file {:?}", dst))
                .write_all(contents.as_ref())
                .unwrap_or_else(|_| panic!("couldn't write to file {:?}: {:?}", dst, contents));
        }

        if self.git {
            use assert_cmd::prelude::*;
            use std::process::Command;

            Command::new("git")
                .arg("init")
                .current_dir(&self.root)
                .assert()
                .success();

            if let Some(ref branch) = self.branch {
                // Create dummy content in "main" branch to aid testing

                fs::File::create(self.root.join("dummy.txt"))
                    .expect("Failed to create dummy")
                    .write_all(b"main dummy")
                    .expect("Couldn't write out dummy text");

                Command::new("git")
                    .arg("add")
                    .arg("dummy.txt")
                    .current_dir(&self.root)
                    .assert()
                    .success();

                Command::new("git")
                    .arg("commit")
                    .arg("--message")
                    .arg("initial main commit")
                    .current_dir(&self.root)
                    .assert()
                    .success();

                self.rename_branch_to_main();

                Command::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg(branch)
                    .current_dir(&self.root)
                    .assert()
                    .success();
            }

            Command::new("git")
                .arg("add")
                .arg("--all")
                .current_dir(&self.root)
                .assert()
                .success();

            self.submodules.iter().for_each(|(d, m)| {
                Command::new("git")
                    .arg("submodule")
                    .arg("add")
                    .arg(&m)
                    .arg(&d)
                    .current_dir(&self.root)
                    .assert()
                    .success();
            });

            Command::new("git")
                .arg("commit")
                .arg("--message")
                .arg("initial commit")
                .current_dir(&self.root)
                .assert()
                .success();

            if self.branch.is_some() {
                Command::new("git")
                    .arg("checkout")
                    .arg("main")
                    .current_dir(&self.root)
                    .assert()
                    .success();
            } else {
                self.rename_branch_to_main();
            }
        }

        Project { root: self.root }
    }
}
