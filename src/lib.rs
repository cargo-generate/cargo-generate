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
mod filenames;
mod git;
mod ignore_me;
mod include_exclude;
mod interactive;
mod log;
mod progressbar;
mod project_variables;
mod template;
mod template_variables;

use anyhow::{anyhow, bail, Context, Result};
use config::{Config, ConfigValues, CONFIG_FILE_NAME};
use console::style;
use favorites::{list_favorites, resolve_favorite_args};
use std::{
    borrow::Borrow,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;
use tempfile::TempDir;

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
            _ => Err(anyhow!("Must be one of 'git' or 'none'")),
        }
    }
}

pub fn generate(mut args: Args) -> Result<()> {
    if args.list_favorites {
        return list_favorites(&args);
    }

    resolve_favorite_args(&mut args)?;

    let project_name = resolve_project_name(&args)?;
    let project_dir = create_project_dir(&project_name, args.force)?;

    let (template_base_dir, branch) = clone_git_template_into_temp(&args)?;
    let template_folder = resolve_template_dir(&template_base_dir, &args)?;

    let template_values = args
        .template_values_file
        .as_ref()
        .map(|p| Path::new(p))
        .map_or(Ok(Default::default()), |path| get_config_file_values(path))?;

    let template_config = Config::from_path(
        &locate_template_file(CONFIG_FILE_NAME, &template_base_dir, &args.subfolder).ok(),
    )?;

    expand_template(
        &project_name,
        &template_folder,
        &template_values,
        template_config,
        &args,
    )?;

    copy_dir_all(&template_folder, &project_dir)?;

    initialize_vcs(args, &project_dir, branch)?;

    println!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(&project_dir.display()).underlined()
    );
    Ok(())
}

fn initialize_vcs(args: Args, project_dir: &Path, branch: String) -> Result<()> {
    match args.vcs {
        Vcs::None => {}
        Vcs::Git => {
            git::init(project_dir, &branch)?;
        }
    };
    Ok(())
}

fn resolve_project_name(args: &Args) -> Result<ProjectName> {
    match args.name {
        Some(ref n) => Ok(ProjectName::new(n)),
        None if !args.silent => Ok(ProjectName::new(interactive::name()?)),
        None => Err(anyhow!(
            "{} {} {}",
            emoji::ERROR,
            style("Project Name Error:").bold().red(),
            style("Option `--silent` provided, but project name was not set. Please use `--name`.")
                .bold()
                .red(),
        )),
    }
}

fn resolve_template_dir(template_base_dir: &TempDir, args: &Args) -> Result<PathBuf> {
    match &args.subfolder {
        Some(subfolder) => {
            let template_base_dir = fs::canonicalize(template_base_dir.path())?;
            let template_dir = fs::canonicalize(template_base_dir.join(subfolder))?;

            if !template_dir.starts_with(&template_base_dir) {
                return Err(anyhow!(
                    "{} {} {}",
                    emoji::ERROR,
                    style("Subfolder Error:").bold().red(),
                    style("Invalid subfolder. Must be part of the template folder structure.")
                        .bold()
                        .red(),
                ));
            }
            if !template_dir.is_dir() {
                return Err(anyhow!(
                    "{} {} {}",
                    emoji::ERROR,
                    style("Subfolder Error:").bold().red(),
                    style("The specified subfolder must be a valid folder.")
                        .bold()
                        .red(),
                ));
            }

            println!(
                "{} {} `{}`{}",
                emoji::WRENCH,
                style("Using template subfolder").bold(),
                style(subfolder).bold().yellow(),
                style("...").bold()
            );
            Ok(template_dir)
        }
        None => Ok(template_base_dir.path().to_owned()),
    }
}

fn clone_git_template_into_temp(args: &Args) -> Result<(TempDir, String)> {
    let git_clone_dir = tempfile::tempdir()?;

    let remote = args
        .git
        .clone()
        .with_context(|| "Missing option git, or a favorite")?;

    let git_config = GitConfig::new_abbr(
        remote.into(),
        args.branch.to_owned(),
        args.ssh_identity.clone(),
    )?;

    let branch = git::create(git_clone_dir.path(), git_config).map_err(|e| {
        anyhow!(
            "{} {} {}",
            emoji::ERROR,
            style("Git Error:").bold().red(),
            style(e).bold().red(),
        )
    })?;

    Ok((git_clone_dir, branch))
}

pub(crate) fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else if ty.is_file() {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            bail!(
                "{} {}",
                crate::emoji::WARN,
                style("Symbolic links not supported").bold().red(),
            )
        }
    }
    Ok(())
}

fn locate_template_file<T>(
    name: &str,
    template_folder: T,
    subfolder: &Option<String>,
) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let template_folder = template_folder.as_ref().to_path_buf();
    let mut search_folder = subfolder
        .as_ref()
        .map_or_else(|| template_folder.to_owned(), |s| template_folder.join(s));
    loop {
        let file_path = search_folder.join(name.borrow());
        if file_path.exists() {
            return Ok(file_path);
        }
        if search_folder == template_folder {
            bail!("File not found within template");
        }
        search_folder = search_folder
            .parent()
            .ok_or_else(|| anyhow!("Reached root folder"))?
            .to_path_buf();
    }
}

fn get_config_file_values(path: &Path) -> Result<HashMap<String, toml::Value>> {
    match fs::read_to_string(path) {
        Ok(ref contents) => toml::from_str::<ConfigValues>(contents)
            .map(|v| v.values)
            .map_err(|e| e.into()),
        Err(e) => bail!(
            "{} {} {}",
            emoji::ERROR,
            style("Values File Error:").bold().red(),
            style(e).bold().red(),
        ),
    }
}

fn create_project_dir(name: &ProjectName, force: bool) -> Result<PathBuf> {
    let dir_name = if force {
        name.raw()
    } else {
        rename_warning(name);
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
        Err(anyhow!(
            "{} {}",
            emoji::ERROR,
            style("Target directory already exists, aborting!")
                .bold()
                .red()
        ))
    } else {
        Ok(project_dir)
    }
}

fn expand_template(
    name: &ProjectName,
    dir: &Path,
    template_values: &HashMap<String, toml::Value>,
    template_config: Option<Config>,
    args: &Args,
) -> Result<()> {
    let crate_type: CrateType = args.into();
    let template = template::substitute(name, &crate_type, template_values, args.force)?;
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

    Ok(())
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
