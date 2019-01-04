extern crate cargo as upstream;
extern crate console;
extern crate dialoguer;
extern crate git2;
extern crate heck;
extern crate ignore;
extern crate indicatif;
extern crate liquid;
extern crate quicli;
extern crate regex;
extern crate remove_dir_all;
extern crate tempfile;
extern crate url;
extern crate walkdir;
extern crate structopt;

mod cargo;
mod emoji;
mod git;
mod ignoreme;
mod interactive;
mod progressbar;
mod projectname;
mod template;

use console::style;
use crate::git::GitConfig;
use crate::projectname::ProjectName;
use quicli::prelude::*;
use structopt::StructOpt;
use std::env;
use std::path::PathBuf;

/// Generate a new Cargo project from a given template
///
/// Right now, only git repositories can be used as templates. Just execute
///
/// $ cargo generate --git https://github.com/user/template.git --name foo
///
/// or
///
/// $ cargo gen --git https://github.com/user/template.git --name foo
///
/// and a new Cargo project called foo will be generated.
///
/// TEMPLATES:
///
/// In templates, the following placeholders can be used:
///
/// - `project-name`: Name of the project, in dash-case
///
/// - `crate_name`: Name of the project, but in a case valid for a Rust
///   identifier, i.e., snake_case
///
/// - `authors`: Author names, taken from usual environment variables (i.e.
///   those which are also used by Cargo and git)
#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Cli {
    #[structopt(name = "generate")]
    Generate(Args),
    #[structopt(name = "gen")]
    Gen(Args),
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long = "git")]
    git: String,
    // Branch to use when installing from git
    #[structopt(long = "branch")]
    branch: Option<String>,
    #[structopt(long = "name", short = "n")]
    name: Option<String>,
    /// Enforce to create a new project without case conversion of project name
    #[structopt(long = "force", short = "f")]
    force: bool,
}

pub fn generate(_cli: Cli) -> Result<()> {
    let args: Args = match Cli::from_args() {
        Cli::Generate(args) => args,
        Cli::Gen(args) => args,
    };

    let name = match &args.name {
        Some(ref n) => ProjectName::new(n),
        None => ProjectName::new(&interactive::name()?),
    };

    rename_warning(&name);
    create_git(args, &name)?;

    Ok(())
}

fn create_git(args: Args, name: &ProjectName) -> Result<()> {
    let force = args.force;
    let config = GitConfig::new(args.git, args.branch)?;
    if let Some(dir) = &create_project_dir(&name, force) {
        match git::create(dir, config) {
            Ok(_) => git::remove_history(dir).unwrap_or(progress(name, dir, force)?),
            Err(e) => println!(
                "{} {} {}",
                emoji::ERROR,
                style("Git Error:").bold().red(),
                style(e).bold().red(),
            ),
        };
    } else {
        println!(
            "{} {}",
            emoji::ERROR,
            style("Target directory already exists, aborting!")
                .bold()
                .red(),
        );
    }
    Ok(())
}

fn create_project_dir(name: &ProjectName, force: bool) -> Option<PathBuf> {
    let dir_name = if force { name.raw() } else { name.kebab_case() };
    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&dir_name);

    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Creating project called").bold(),
        style(&name.kebab_case()).bold().yellow(),
        style("...").bold()
    );

    if project_dir.exists() {
        None
    } else {
        Some(project_dir)
    }
}

fn progress(name: &ProjectName, dir: &PathBuf, force: bool) -> Result<()> {
    let template = template::substitute(name, force)?;

    let pbar = progressbar::new();
    pbar.tick();

    template::walk_dir(dir, template, pbar)?;

    git::init(dir)?;

    ignoreme::remove_uneeded_files(dir);

    gen_success(dir);

    Ok(())
}

fn gen_success(dir: &PathBuf) {
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
        println!(
            "{} {} `{}` {} `{}`{}",
            emoji::WARN,
            style("Renaming project called").bold(),
            style(&name.user_input).bold().yellow(),
            style("to").bold(),
            style(&name.kebab_case()).bold().green(),
            style("...").bold()
        );
    }
}
