use assert_cmd::prelude::*;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use tempfile::TempDir;

pub fn binary() -> CargoGenerateArgBuilder {
    CargoGenerateArgBuilder::new()
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
            .unwrap_or_else(|_| panic!("couldn't open file {path:?}"))
            .read_to_string(&mut ret)
            .unwrap_or_else(|_| panic!("couldn't read file {path:?}"));

        ret
    }

    pub fn path(&self) -> &Path {
        self.root.path()
    }

    /// returns the path of the generated project aka target path
    pub fn target_path(&self, name: impl AsRef<str>) -> PathBuf {
        self.path().join(name.as_ref())
    }

    pub fn exists(&self, path: &str) -> bool {
        self.path().join(path).exists()
    }
}

pub struct CargoGenerateArgBuilder(Command);

impl CargoGenerateArgBuilder {
    pub fn new() -> Self {
        let mut builder = Self(Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap());
        builder.0.arg("generate");

        builder
    }

    /// wrapper for `--name <name>` cli argument
    pub fn arg_name(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.0.arg("--name").arg(name);

        self
    }

    /// wrapper for `--branch <name>` cli argument
    pub fn arg_branch(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.0.arg("--branch").arg(name);

        self
    }

    /// wrapper for `--git <name>` cli argument
    pub fn arg_git(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.0.arg("--git").arg(name);

        self
    }

    /// proxy for `Command::arg`
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.0.arg(arg);
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        self.0.args(args);
        self
    }

    /// proxy for `Command::current_dir` also it consumes self and returns the inner Command
    pub fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Command {
        self.0.current_dir(path)
    }
}
