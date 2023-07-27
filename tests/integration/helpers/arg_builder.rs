use assert_cmd::prelude::*;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

pub fn binary() -> CargoGenerateArgBuilder {
    CargoGenerateArgBuilder::new()
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
        self.arg("--name").arg(name)
    }

    /// wrapper for `--branch <name>` cli argument
    pub fn arg_branch(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--branch").arg(name)
    }

    /// wrapper for `--git <name>` cli argument
    pub fn arg_git(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--git").arg(name)
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
