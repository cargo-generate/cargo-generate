use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use structopt::StructOpt;

use crate::git;

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Cli {
    #[structopt(name = "generate", visible_alias = "gen")]
    Generate(Args),
}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// List defined favorite templates from the config
    #[structopt(
        long,
        conflicts_with = "git",
        conflicts_with = "subfolder",
        conflicts_with = "path",
        conflicts_with = "branch",
        conflicts_with = "name",
        conflicts_with = "force",
        conflicts_with = "template_values_file",
        conflicts_with = "silent",
        conflicts_with = "vcs",
        conflicts_with = "lib",
        conflicts_with = "bin",
        conflicts_with = "ssh_identity",
        conflicts_with = "define",
        conflicts_with = "init"
    )]
    pub list_favorites: bool,

    /// Generate a favorite template as defined in the config. In case the favorite is undefined,
    /// use in place of the `--git` option, otherwise specifies the subfolder
    #[structopt(name = "favorite|git|subfolder")]
    pub favorite: Option<String>,

    /// Specifies a subfolder within the template repository to be used as the actual template.
    #[structopt()]
    pub subfolder: Option<String>,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    ///
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[structopt(short, long, conflicts_with = "subfolder")]
    pub git: Option<String>,

    /// Local path to copy the template from. Can not be specified together with --git.
    #[structopt(
        short,
        long,
        conflicts_with = "git",
        conflicts_with = "favorite",
        conflicts_with = "subfolder"
    )]
    pub path: Option<PathBuf>,

    /// Branch to use when installing from git
    #[structopt(short, long)]
    pub branch: Option<String>,

    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[structopt(long, short)]
    pub name: Option<String>,

    /// Don't convert the project name to kebab-case before creating the directory.
    /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
    #[structopt(long, short)]
    pub force: bool,

    /// Enables more verbose output.
    #[structopt(long, short)]
    pub verbose: bool,

    /// Pass template values through a file
    /// Values should be in the format `key=value`, one per line
    #[structopt(long)]
    pub template_values_file: Option<String>,

    /// If silent mode is set all variables will be
    /// extracted from the template_values_file.
    /// If a value is missing the project generation will fail
    #[structopt(long, short, requires("name"))]
    pub silent: bool,

    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
    #[structopt(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Specify the VCS used to initialize the generated template.
    #[structopt(long, default_value = "git")]
    pub vcs: Vcs,

    /// Populates a template variable `crate_type` with value `"lib"`
    #[structopt(long, conflicts_with = "bin")]
    pub lib: bool,

    /// Populates a template variable `crate_type` with value `"bin"`
    #[structopt(long, conflicts_with = "lib")]
    pub bin: bool,

    /// Use a different ssh identity
    #[structopt(short = "i", long = "identity", parse(from_os_str))]
    pub ssh_identity: Option<PathBuf>,

    /// Define a value for use during template expansion
    #[structopt(long, short, number_of_values = 1)]
    pub define: Vec<String>,

    /// Generate the template directly into the current dir. No subfolder will be created and no vcs is initialized.
    #[structopt(long)]
    pub init: bool,

    /// Will enforce a fresh git init on the generated project
    #[structopt(long)]
    pub force_git_init: bool,
}

#[derive(Debug, StructOpt, Clone, Copy)]
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
    pub fn initialize(&self, project_dir: &Path, branch: String) -> Result<()> {
        match self {
            Self::None => {}
            Self::Git => {
                git::init(project_dir, &branch)?;
            }
        };
        Ok(())
    }
}
