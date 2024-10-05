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

    /// wrapper for `--init` cli flag
    pub fn flag_init(&mut self) -> &mut Self {
        self.arg("--init")
    }

    /// wrapper for `--name <name>` cli argument
    pub fn arg_name(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--name").arg(name)
    }

    /// wrapper for `--branch <name>` cli argument
    pub fn arg_branch(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--branch").arg(name)
    }

    /// wrapper for `--revision <name>` cli argument
    pub fn arg_revision(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--rev").arg(name)
    }

    /// wrapper for `--git <name>` cli argument
    pub fn arg_git(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--git").arg(name)
    }

    /// wrapper for `--path <name>` cli argument
    pub fn arg_path(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--path").arg(name)
    }

    /// wrapper for `--gitconfig <file>` cli argument
    pub fn arg_gitconfig(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--gitconfig").arg(name)
    }

    #[allow(dead_code)]
    /// wrapper for `--identity <file>` cli argument
    pub fn arg_identity(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.arg("--identity").arg(name)
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
