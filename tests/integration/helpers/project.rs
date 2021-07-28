use assert_cmd::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::str;

use tempfile::TempDir;

pub fn binary() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[derive(Debug)]
pub struct Project {
    pub(crate) root: TempDir,
}

impl Project {
    pub fn read(&self, path: &str) -> String {
        let mut ret = String::new();
        let path = self.path().join(path);
        File::open(&path)
            .unwrap_or_else(|_| panic!("couldn't open file {:?}", path))
            .read_to_string(&mut ret)
            .unwrap_or_else(|_| panic!("couldn't read file {:?}", path));

        ret
    }

    pub fn path(&self) -> &Path {
        self.root.path()
    }

    pub fn exists(&self, path: &str) -> bool {
        self.path().join(path).exists()
    }
}
