extern crate assert_cmd;
extern crate git2;
extern crate predicates;
extern crate tempfile;

// actual tests

mod basics;

// helpers below

mod helpers {
    use std::env;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::str;
    use std::sync::atomic::*;

    use git2;

    static CNT: AtomicUsize = ATOMIC_USIZE_INIT;
    thread_local!(static IDX: usize = CNT.fetch_add(1, Ordering::SeqCst));

    pub struct ProjectBuilder {
        files: Vec<(String, String)>,
        root: PathBuf,
        git: bool,
    }

    pub struct Project {
        root: PathBuf,
    }

    pub fn dir(name: &str) -> ProjectBuilder {
        ProjectBuilder {
            files: Vec::new(),
            root: root(name),
            git: false,
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
        return me;
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

        pub fn build(self) -> Project {
            drop(fs::remove_dir_all(&self.root));
            fs::create_dir_all(&self.root)
                .expect(&format!("couldn't create {:?} directory", self.root));

            for &(ref file, ref contents) in self.files.iter() {
                let dst = self.root.join(file);

                fs::create_dir_all(
                    dst.parent()
                        .expect(&format!("couldn't find parent dir of {:?}", dst)),
                ).expect(&format!("couldn't create {:?} directory", dst.parent()));

                fs::File::create(&dst)
                    .expect(&format!("couldn't create file {:?}", dst))
                    .write_all(contents.as_ref())
                    .expect(&format!("couldn't write to file {:?}: {:?}", dst, contents));
            }

            if self.git {
                use assert_cmd::prelude::*;
                use std::process::Command;

                Command::new("git")
                    .arg("init")
                    .current_dir(&self.root)
                    .assert()
                    .success();
                Command::new("git")
                    .arg("add")
                    .arg("--all")
                    .current_dir(&self.root)
                    .assert()
                    .success();
                Command::new("git")
                    .arg("commit")
                    .arg("--message")
                    .arg("initial commit")
                    .current_dir(&self.root)
                    .assert()
                    .success();
            }

            Project { root: self.root }
        }
    }

    impl Project {
        pub fn read(&self, path: &str) -> String {
            let mut ret = String::new();
            File::open(self.root.join(path))
                .expect(&format!("couldn't open file {:?}", self.root.join(path)))
                .read_to_string(&mut ret)
                .expect(&format!("couldn't read file {:?}", self.root.join(path)));
            return ret;
        }

        pub fn path(&self) -> &Path {
            &self.root
        }
    }

    impl Drop for Project {
        fn drop(&mut self) {
            drop(fs::remove_dir_all(&self.root));
            drop(fs::remove_dir(&self.root));
        }
    }

}
