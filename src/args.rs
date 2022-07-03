use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use serde::Deserialize;

use crate::git;

#[derive(Parser)]
#[clap(name = "cargo generate")]
#[clap(bin_name = "cargo")]
#[clap(arg_required_else_help(true))]
#[clap(version)]
#[clap(about)]
pub enum Cli {
    #[clap(name = "generate", visible_alias = "gen")]
    Generate(GenerateArgs),
}

#[derive(Clone, Debug, Args)]
#[clap(arg_required_else_help(true))]
#[clap(version)]
#[clap(about)]
pub struct GenerateArgs {
    #[clap(flatten)]
    pub template_path: TemplatePath,

    /// List defined favorite templates from the config
    #[clap(
        long,
        action,
        conflicts_with_all(&[
            "git", "path", "subfolder", "branch",
            "name",
            "force",
            "silent",
            "vcs",
            "lib",
            "bin",
            "define",
            "init",
            "template-values-file",
            "ssh-identity"
        ])
    )]
    pub list_favorites: bool,

    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[clap(long, short, value_parser)]
    pub name: Option<String>,

    /// Don't convert the project name to kebab-case before creating the directory.
    /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
    #[clap(long, short, action)]
    pub force: bool,

    /// Enables more verbose output.
    #[clap(long, short, action)]
    pub verbose: bool,

    /// Pass template values through a file
    /// Values should be in the format `key=value`, one per line
    #[clap(long, value_parser)]
    pub template_values_file: Option<String>,

    /// If silent mode is set all variables will be
    /// extracted from the template_values_file.
    /// If a value is missing the project generation will fail
    #[clap(long, short, requires("name"), action)]
    pub silent: bool,

    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
    #[clap(short, long, value_parser)]
    pub config: Option<PathBuf>,

    /// Specify the VCS used to initialize the generated template.
    #[clap(long, value_parser)]
    pub vcs: Option<Vcs>,

    /// Populates template variable `crate_type` with value `"lib"`
    #[clap(long, conflicts_with = "bin", action)]
    pub lib: bool,

    /// Populates a template variable `crate_type` with value `"bin"`
    #[clap(long, conflicts_with = "lib", action)]
    pub bin: bool,

    /// Use a different ssh identity
    #[clap(short = 'i', long = "identity", value_parser)]
    pub ssh_identity: Option<PathBuf>,

    /// Define a value for use during template expansion
    #[clap(long, short, number_of_values = 1, value_parser)]
    pub define: Vec<String>,

    /// Generate the template directly into the current dir. No subfolder will be created and no vcs is initialized.
    #[clap(long, conflicts_with = "destination", action)]
    pub init: bool,

    /// Generate the template directly at the given path.
    #[clap(long, conflicts_with = "init", value_parser)]
    pub destination: Option<PathBuf>,

    /// Will enforce a fresh git init on the generated project
    #[clap(long, action)]
    pub force_git_init: bool,

    /// Allows running system commands without being prompted.
    /// Warning: Setting this flag will enable the template to run arbitrary system commands without user confirmation.
    /// Use at your own risk and be sure to review the template code beforehand.
    #[clap(short, long, action)]
    pub allow_commands: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TemplatePath {
    /// Auto attempt to use as either `--git` or `--favorite`.
    /// If either is specified explicitly, use as subfolder.
    #[clap(required_unless_present_any(&["SpecificPath"]))]
    pub auto_path: Option<String>,

    /// Specifies a subfolder within the template repository to be used as the actual template.
    #[clap()]
    pub subfolder: Option<String>,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    ///
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[clap(short, long, group("SpecificPath"))]
    pub git: Option<String>,

    /// Branch to use when installing from git
    #[clap(short, long)]
    pub branch: Option<String>,

    /// Local path to copy the template from. Can not be specified together with --git.
    #[clap(short, long, group("SpecificPath"))]
    pub path: Option<String>,

    /// Generate a favorite template as defined in the config. In case the favorite is undefined,
    /// use in place of the `--git` option, otherwise specifies the subfolder
    #[clap(long, group("SpecificPath"))]
    pub favorite: Option<String>,
}

impl TemplatePath {
    /// # Panics
    /// Will panic if no path to a template has been set at all,
    /// which is never if Clap is initialized properly.
    pub fn any_path(&self) -> &str {
        self.git
            .as_ref()
            .or(self.path.as_ref())
            .or(self.favorite.as_ref())
            .or(self.auto_path.as_ref())
            .unwrap()
    }

    pub const fn git(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.git.as_ref()
    }

    pub const fn branch(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.branch.as_ref()
    }

    pub const fn path(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.path.as_ref()
    }

    pub const fn favorite(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.favorite.as_ref()
    }

    pub const fn auto_path(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.auto_path.as_ref()
    }

    pub const fn subfolder(&self) -> Option<&(impl AsRef<str> + '_)> {
        if self.git.is_some() || self.path.is_some() || self.favorite.is_some() {
            self.auto_path.as_ref()
        } else {
            self.subfolder.as_ref()
        }
    }
}

#[derive(Debug, Parser, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Vcs {
    None,
    Git,
}

impl FromStr for Vcs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NONE" => Ok(Self::None),
            "GIT" => Ok(Self::Git),
            _ => Err(anyhow!("Must be one of 'git' or 'none'")),
        }
    }
}

impl Vcs {
    pub fn initialize(&self, project_dir: &Path, branch: String, force: bool) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Git => git::init(project_dir, &branch, force)
                .map(|_| ())
                .map_err(anyhow::Error::from),
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
