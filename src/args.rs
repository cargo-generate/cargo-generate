use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use serde::Deserialize;

use crate::git;

/// Styles from <https://github.com/rust-lang/cargo/blob/master/src/cargo/util/style.rs>
mod style {
    use anstyle::*;
    use clap::builder::Styles;

    const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
    const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

    pub const STYLES: Styles = {
        Styles::styled()
            .header(HEADER)
            .usage(USAGE)
            .literal(LITERAL)
            .placeholder(PLACEHOLDER)
            .error(ERROR)
            .valid(VALID)
            .invalid(INVALID)
            .error(ERROR)
    };
}

mod heading {
    pub const GIT_PARAMETERS: &str = "Git Parameters";
    pub const TEMPLATE_SELECTION: &str = "Template Selection";
    pub const OUTPUT_PARAMETERS: &str = "Output Parameters";
}

#[derive(Parser)]
#[command(
    name = "cargo generate",
    bin_name = "cargo",
    arg_required_else_help(true),
    version,
    about,
    next_line_help(false),
    styles(style::STYLES)
)]
pub enum Cli {
    #[command(name = "generate", visible_alias = "gen")]
    Generate(GenerateArgs),
}

#[derive(Clone, Debug, Args)]
#[command(arg_required_else_help(true), version, about)]
pub struct GenerateArgs {
    #[command(flatten)]
    pub template_path: TemplatePath,

    /// List defined favorite templates from the config
    #[arg(
        long,
        action,
        group("SpecificPath"),
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
            "template_values_file",
            "ssh_identity",
            "test",
        ])
    )]
    pub list_favorites: bool,

    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[arg(long, short, value_parser, help_heading = heading::OUTPUT_PARAMETERS)]
    pub name: Option<String>,

    /// Don't convert the project name to kebab-case before creating the directory. Note that cargo
    /// generate won't overwrite an existing directory, even if `--force` is given.
    #[arg(long, short, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub force: bool,

    /// Enables more verbose output.
    #[arg(long, short, action)]
    pub verbose: bool,

    /// Pass template values through a file. Values should be in the format `key=value`, one per
    /// line
    #[arg(long="values-file", value_parser, alias="template-values-file", value_name="FILE", help_heading = heading::OUTPUT_PARAMETERS)]
    pub template_values_file: Option<String>,

    /// If silent mode is set all variables will be extracted from the template_values_file. If a
    /// value is missing the project generation will fail
    #[arg(long, short, requires("name"), action)]
    pub silent: bool,

    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or
    /// $HOME/.cargo/cargo-generate
    #[arg(short, long, value_parser)]
    pub config: Option<PathBuf>,

    /// Specify the VCS used to initialize the generated template.
    #[arg(long, value_parser, help_heading = heading::OUTPUT_PARAMETERS)]
    pub vcs: Option<Vcs>,

    /// Populates template variable `crate_type` with value `"lib"`
    #[arg(long, conflicts_with = "bin", action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub lib: bool,

    /// Populates a template variable `crate_type` with value `"bin"`
    #[arg(long, conflicts_with = "lib", action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub bin: bool,

    /// Use a different ssh identity
    #[arg(short = 'i', long = "identity", value_parser, value_name="IDENTITY", help_heading = heading::GIT_PARAMETERS)]
    pub ssh_identity: Option<PathBuf>,

    /// Use a different gitconfig file, if omitted the usual $HOME/.gitconfig will be used
    #[arg(long = "gitconfig", value_parser, value_name="GITCONFIG_FILE", help_heading = heading::GIT_PARAMETERS)]
    pub gitconfig: Option<PathBuf>,

    /// Define a value for use during template expansion. E.g `--define foo=bar`
    #[arg(long, short, number_of_values = 1, value_parser, help_heading = heading::OUTPUT_PARAMETERS)]
    pub define: Vec<String>,

    /// Generate the template directly into the current dir. No subfolder will be created and no vcs
    /// is initialized.
    #[arg(long, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub init: bool,

    /// Generate the template directly at the given path.
    #[arg(long, value_parser, value_name="PATH", help_heading = heading::OUTPUT_PARAMETERS)]
    pub destination: Option<PathBuf>,

    /// Will enforce a fresh git init on the generated project
    #[arg(long, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub force_git_init: bool,

    /// Allows running system commands without being prompted. Warning: Setting this flag will
    /// enable the template to run arbitrary system commands without user confirmation. Use at your
    /// own risk and be sure to review the template code beforehand.
    #[arg(short, long, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub allow_commands: bool,

    /// Allow the template to overwrite existing files in the destination.
    #[arg(short, long, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub overwrite: bool,

    /// Skip downloading git submodules (if there are any)
    #[arg(long, action, help_heading = heading::GIT_PARAMETERS)]
    pub skip_submodules: bool,

    /// All args after "--" on the command line.
    #[arg(skip)]
    pub other_args: Option<Vec<String>>,
}

impl Default for GenerateArgs {
    fn default() -> Self {
        Self {
            template_path: TemplatePath::default(),
            list_favorites: false,
            name: None,
            force: false,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: None,
            lib: true,
            bin: false,
            ssh_identity: None,
            gitconfig: None,
            define: Vec::default(),
            init: false,
            destination: None,
            force_git_init: false,
            allow_commands: false,
            overwrite: false,
            skip_submodules: false,
            other_args: None,
        }
    }
}

#[derive(Default, Debug, Clone, Args)]
pub struct TemplatePath {
    /// Auto attempt to use as either `--git` or `--favorite`. If either is specified explicitly,
    /// use as subfolder.
    #[arg(required_unless_present_any(&["SpecificPath"]))]
    pub auto_path: Option<String>,

    /// Specifies the subfolder within the template repository to be used as the actual template.
    #[arg()]
    pub subfolder: Option<String>,

    /// Expand $CWD as a template, then run `cargo test` on the expansion (set
    /// $CARGO_GENERATE_TEST_CMD to override test command).
    ///
    /// Any arguments given after the `--test` argument, will be used as arguments for the test
    /// command.
    #[arg(long, action, group("SpecificPath"))]
    pub test: bool,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    ///
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[arg(short, long, group("SpecificPath"), help_heading = heading::TEMPLATE_SELECTION)]
    pub git: Option<String>,

    /// Branch to use when installing from git
    #[arg(short, long, conflicts_with_all = ["revision", "tag"], help_heading = heading::GIT_PARAMETERS)]
    pub branch: Option<String>,

    /// Tag to use when installing from git
    #[arg(short, long, conflicts_with_all = ["revision", "branch"], help_heading = heading::GIT_PARAMETERS)]
    pub tag: Option<String>,

    /// Git revision to use when installing from git (e.g. a commit hash)
    #[arg(short, long, conflicts_with_all = ["tag", "branch"], alias = "rev", help_heading = heading::GIT_PARAMETERS)]
    pub revision: Option<String>,

    /// Local path to copy the template from. Can not be specified together with --git.
    #[arg(short, long, group("SpecificPath"), help_heading = heading::TEMPLATE_SELECTION)]
    pub path: Option<String>,

    /// Generate a favorite template as defined in the config. In case the favorite is undefined,
    /// use in place of the `--git` option, otherwise specifies the subfolder
    #[arg(long, group("SpecificPath"), help_heading = heading::TEMPLATE_SELECTION)]
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

    pub const fn tag(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.tag.as_ref()
    }

    pub const fn revision(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.revision.as_ref()
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
    pub fn initialize(&self, project_dir: &Path, branch: Option<&str>, force: bool) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Git => git::init(project_dir, branch, force)
                .map(|_| ())
                .map_err(anyhow::Error::from),
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
