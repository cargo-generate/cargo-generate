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
mod args;
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

pub use args::*;

use anyhow::{anyhow, bail, Context, Result};
use config::{Config, CONFIG_FILE_NAME};
use console::style;
use favorites::{list_favorites, resolve_favorite_args};
use std::{
    borrow::Borrow,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use crate::{
    app_config::{app_config_path, AppConfig},
    template_variables::{CrateType, ProjectName},
};
use crate::{git::GitConfig, template_variables::resolve_template_values};

pub fn generate(mut args: Args) -> Result<()> {
    let path = &app_config_path(&args.config)?;
    let app_config = AppConfig::from_path(path)?;

    if args.list_favorites {
        return list_favorites(&app_config, &args);
    }

    resolve_favorite_args(&app_config, &mut args)?;

    let project_name = resolve_project_name(&args)?;
    let project_dir = resolve_project_dir(&project_name, args.force)?;

    let (template_base_dir, template_folder, branch) = prepare_local_template(&args)?;

    let template_config = Config::from_path(
        &locate_template_file(CONFIG_FILE_NAME, &template_base_dir, &args.subfolder).ok(),
    )?;

    let template_values = resolve_template_values(&args)?;

    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Creating project called").bold(),
        style(project_dir.display()).bold().yellow(),
        style("...").bold()
    );

    expand_template(
        &project_name,
        &template_folder,
        &template_values,
        template_config,
        &args,
    )?;
    copy_dir_all(&template_folder, &project_dir)?;
    args.vcs.initialize(&project_dir, branch)?;

    println!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(&project_dir.display()).underlined()
    );
    Ok(())
}

fn prepare_local_template(args: &Args) -> Result<(TempDir, PathBuf, String), anyhow::Error> {
    let (template_base_dir, template_folder, branch) = match (&args.git, &args.path) {
        (Some(_), None) => {
            let (template_base_dir, branch) = clone_git_template_into_temp(args)?;
            let template_folder = resolve_template_dir(&template_base_dir, args)?;
            (template_base_dir, template_folder, branch)
        }
        (None, Some(_)) => {
            let template_base_dir = copy_path_template_into_temp(args)?;
            let branch = args.branch.clone().unwrap_or_else(|| String::from("main"));
            let template_folder = template_base_dir.path().into();
            (template_base_dir, template_folder, branch)
        }
        _ => bail!(
            "{} {} {} {} {}",
            emoji::ERROR,
            style("Please specify either").bold(),
            style("--git <repo>").bold().yellow(),
            style("or").bold(),
            style("--path <path>").bold().yellow(),
        ),
    };
    Ok((template_base_dir, template_folder, branch))
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

fn copy_path_template_into_temp(args: &Args) -> Result<TempDir> {
    let path_clone_dir = tempfile::tempdir()?;
    copy_dir_all(
        args.path
            .as_ref()
            .with_context(|| "Missing option git, path or a favorite")?,
        path_clone_dir.path(),
    )?;
    Ok(path_clone_dir)
}

fn clone_git_template_into_temp(args: &Args) -> Result<(TempDir, String)> {
    let git_clone_dir = tempfile::tempdir()?;

    let remote = args
        .git
        .clone()
        .with_context(|| "Missing option git, path or a favorite")?;

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

fn resolve_project_dir(name: &ProjectName, force: bool) -> Result<PathBuf> {
    let dir_name = if force {
        name.raw()
    } else {
        rename_warning(name);
        name.kebab_case()
    };
    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&dir_name);

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
