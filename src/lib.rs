extern crate console;
extern crate dialoguer;
extern crate git2;
extern crate heck;
extern crate indicatif;
extern crate liquid;
extern crate quicli;
extern crate regex;
extern crate remove_dir_all;
extern crate walkdir;

mod cargo;
mod emoji;
mod git;
pub mod interactive;
mod progressbar;
pub mod projectname;
mod template;

use console::style;
use projectname::ProjectName;
use quicli::prelude::*;
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
    #[structopt(long = "name", short = "n")]
    name: Option<String>,
}

///Takes the command line arguments and starts generating the project
pub fn generate(_cli: Cli) {
    let args: Args = match Cli::from_args() {
        Cli::Generate(args) => args,
        Cli::Gen(args) => args,
    };

    let name = match &args.name {
        Some(ref n) => ProjectName::new(n),
        None => ProjectName::new(&interactive::name().unwrap()),
    };

    create_git(args, &name);
}

pub fn create_git(args: Args, name: &ProjectName) {
    if let Some(dir) = &create_project_dir(&name) {
        match git::create(dir, args) {
            Ok(_) => git::remove_history(dir).unwrap_or(progress(name, dir)),
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
}

fn create_project_dir(name: &ProjectName) -> Option<PathBuf> {
    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Trying to create project called").bold(),
        style(name.kebab_case()).bold().yellow(),
        style("...").bold()
    );

    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(name.kebab_case());

    if project_dir.exists() {
        None
    } else {
        Some(project_dir)
    }
}

//TODO: better error handling for progress?
fn progress(name: &ProjectName, dir: &PathBuf) {
    let template = template::substitute(name).expect("Error: Can't substitute the given name.");

    let pbar = progressbar::new();
    pbar.tick();

    template::walk_dir(dir, template, pbar).expect("Error: Can't walk the directory");

    git::init(dir).expect("Error: Can't init git repo");

    gen_success(dir);
}

fn gen_success(dir: &PathBuf) {
    let dir_string = dir.to_str().unwrap(); //unwrap is safe here
    println!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(dir_string).underlined()
    );
}
