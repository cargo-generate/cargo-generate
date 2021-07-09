//! Generate a new Cargo project from a given template
//!
//! Right now, only git repositories can be used as templates. Just execute
//!
//! $ cargo generate --git https://github.com/user/template.git --name foo
//!
//! or
//!
//! $ cargo gen --git https://github.com/user/template.git --name foo
//!
//! and a new Cargo project called foo will be generated.
//!
//! TEMPLATES:
//!
//! In templates, the following placeholders can be used:
//!
//! - `project-name`: Name of the project, in dash-case
//!
//! - `crate_name`: Name of the project, but in a case valid for a Rust
//!   identifier, i.e., snake_case
//!
//! - `authors`: Author names, taken from usual environment variables (i.e.
//!   those which are also used by Cargo and git)
//!
//! The template author can define their own placeholders in their
//! `cargo-generate.toml` file. This looks like the following:
//!
//! ```toml
//!
//! [placeholders]
//!
//! my-placeholder = { type = "string", prompt = "Hello?", choices = ["hello", "world"], default = "hello", regex = "*" }
//!
//! use-serde = { type = "bool", prompt = "Add serde support?", default = false }
//!
//! ```
//!
//! The user of the template will then be asked the the question in "prompt", and must accept the
//! default value (if provided) or enter a custom value which will be checked against "choices" (if
//! provided) and regex (if provided).
//!
//! The placeholder map supports the following keys:
//!
//! `type` (required): Must be "string" or "bool"
//!
//! `prompt` (required): A string containing the question to be asked to the user
//!
//! `default` (optional): The default value to be used if the user just presses enter. Must be
//! consistent with `type`
//!
//! `choices` (optional; string only): Possible values the user may enter
//!
//! `regex` (optional; string only): Regex to validate the entered string
//!
//! For automation purposes the user of the template may provide provide a file containing the
//! values for the keys in the template by using the `--template-values-file` flag.
//!
//! The file should be a toml file containing the following (for the example template provided above):
//!
//! ```toml
//!
//! [values]
//!
//! my-placeholder = "world"
//!
//! use-serde = true
//!
//! ```
//!
//! If a key is missing in this file, the user will be requested to provide the entry manually. If
//! a key in this file is not part of the original template it will be ignored.
//!
//! To ensure that no interaction will be requested to the user use the `--silent` flag. Then, if a
//! template key is missing an error will be returned and the project generation will fail.
//!
//! Notice: `project-name` and `crate_name` can't be overriden through this file and must be
//! provided through the `--name` flag.
//!
//! `os-arch` and `authors` also can't be overriden and are derived from the environment.

mod app_config;
mod config;
mod emoji;
mod favorites;
mod git;
mod ignore_me;
mod include_exclude;
mod interactive;
mod log;
mod progressbar;
mod project_variables;
mod template;
mod template_variables;

use anyhow::{Context, Result};
use config::{Config, ConfigValues, CONFIG_FILE_NAME};
use console::style;
use favorites::{list_favorites, resolve_favorite};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

use crate::git::GitConfig;
use crate::template_variables::{CrateType, ProjectName};

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Cli {
    #[structopt(name = "generate", visible_alias = "gen")]
    Generate(Args),
}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// List defined favorite templates from the config
    #[structopt(long)]
    pub list_favorites: bool,

    /// Generate a favorite template as defined in the config
    pub favorite: Option<String>,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[structopt(short, long)]
    pub git: Option<String>,

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
}

//
#[derive(Debug, StructOpt)]
pub enum Vcs {
    None,
    Git,
}

impl FromStr for Vcs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NONE" => Ok(Vcs::None),
            "GIT" => Ok(Vcs::Git),
            _ => Err(anyhow::anyhow!("Must be one of 'git' or 'none'")),
        }
    }
}

pub fn generate(mut args: Args) -> Result<()> {
    if args.list_favorites {
        return list_favorites(&args);
    }
    resolve_favorite(&mut args)?;

    let name = match args.name {
        Some(ref n) => ProjectName::new(n),
        None if !args.silent => ProjectName::new(interactive::name()?),
        None => anyhow::bail!(
            "{} {} {}",
            emoji::ERROR,
            style("Project Name Error:").bold().red(),
            style("Option `--silent` provided, but project name was not set. Please use `--project-name`.")
                .bold()
                .red(),
        ),
    };

    create_git(args, &name)?;
    Ok(())
}

fn create_git(args: Args, name: &ProjectName) -> Result<()> {
    let force = args.force;
    let template_values = args
        .template_values_file
        .as_ref()
        .map(|p| Path::new(p))
        .map_or(Ok(Default::default()), |path| get_config_file_values(path))?;
    let remote = args
        .git
        .clone()
        .with_context(|| "Missing option git, or a favorite")?;
    let git_config = GitConfig::new_abbr(
        remote.into(),
        args.branch.to_owned(),
        args.ssh_identity.clone(),
    )?;

    if let Some(dir) = &create_project_dir(&name, force) {
        match git::create(dir, git_config) {
            Ok(branch) => {
                progress(name, &template_values, dir, &branch, &args)?;
            }
            Err(e) => anyhow::bail!(
                "{} {} {}",
                emoji::ERROR,
                style("Git Error:").bold().red(),
                style(e).bold().red(),
            ),
        };
    } else {
        anyhow::bail!(
            "{} {}",
            emoji::ERROR,
            style("Target directory already exists, aborting!")
                .bold()
                .red(),
        );
    }
    Ok(())
}

fn get_config_file_values(path: &Path) -> Result<HashMap<String, toml::Value>> {
    match fs::read_to_string(path) {
        Ok(ref contents) => toml::from_str::<ConfigValues>(contents)
            .map(|v| v.values)
            .map_err(|e| e.into()),
        Err(e) => anyhow::bail!(
            "{} {} {}",
            emoji::ERROR,
            style("Values File Error:").bold().red(),
            style(e).bold().red(),
        ),
    }
}

fn create_project_dir(name: &ProjectName, force: bool) -> Option<PathBuf> {
    let dir_name = if force {
        name.raw()
    } else {
        rename_warning(&name);
        name.kebab_case()
    };
    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&dir_name);

    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Creating project called").bold(),
        style(&dir_name).bold().yellow(),
        style("...").bold()
    );

    if project_dir.exists() {
        None
    } else {
        Some(project_dir)
    }
}

fn progress(
    name: &ProjectName,
    template_values: &HashMap<String, toml::Value>,
    dir: &Path,
    branch: &str,
    args: &Args,
) -> Result<()> {
    let crate_type: CrateType = args.into();
    let template = template::substitute(name, &crate_type, template_values, args.force)?;
    let config_path = dir.join(CONFIG_FILE_NAME);
    let template_config = Config::new(config_path)?;
    let template = match template_config.as_ref() {
        None => Ok(template),
        Some(config) => {
            project_variables::fill_project_variables(template, config, args.silent, |slot| {
                interactive::variable(slot)
            })
        }
    }?;
    let mut pbar = progressbar::new();

    ignore_me::remove_unneeded_files(dir, args.verbose);

    template::walk_dir(
        dir,
        template,
        template_config.and_then(|c| c.template),
        &mut pbar,
    )?;

    pbar.join().unwrap();

    match args.vcs {
        Vcs::None => {}
        Vcs::Git => {
            git::init(dir, branch)?;
        }
    }

    gen_success(dir);

    Ok(())
}

fn gen_success(dir: &Path) {
    let dir_string = dir.to_str().unwrap_or("");
    println!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(dir_string).underlined()
    );
}

fn rename_warning(name: &ProjectName) {
    if !name.is_crate_name() {
        info!(
            "{} `{}` {} `{}`{}",
            style("Renaming project called").bold(),
            style(&name.user_input).bold().yellow(),
            style("to").bold(),
            style(&name.kebab_case()).bold().green(),
            style("...").bold()
        );
    }
}
