#[macro_use]
extern crate quicli;
extern crate console;
extern crate dialoguer;
extern crate git2;
extern crate heck;
extern crate indicatif;
extern crate liquid;
extern crate regex;
extern crate remove_dir_all;
extern crate walkdir;

mod cargo;
mod emoji;
mod git;
mod interactive;
mod progressbar;
mod projectname;
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

main!(|_cli: Cli| {
    let args: Args = match Cli::from_args() {
        Cli::Generate(args) => args,
        Cli::Gen(args) => args,
    };

    let name = match &args.name {
        Some(ref n) => ProjectName::new(n),
        None => ProjectName::new(&interactive::name()?),
    };

    let project_dir = create_project_dir(&name);

    create_git(&project_dir, args, &name);

    gen_success(&project_dir);
});

fn create_project_dir(name: &ProjectName) -> PathBuf {
    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Creating project called").bold(),
        style(name.kebab_case()).bold().yellow(),
        style("...").bold()
    );

    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(name.kebab_case());
    /*
    ensure!(
        !project_dir.exists(),
        "Target directory `{}` already exists, aborting.",
        project_dir.display()
    );
    */
    project_dir
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

fn create_git(dir: &PathBuf, args: Args, name: &ProjectName){
    git::create(dir, args).unwrap();
    git::remove_history(dir).unwrap();

    let template = template::substitute(name).unwrap();

    let pbar = progressbar::new();
    pbar.tick();

    template::walk_dir(dir, template, pbar).unwrap();

    git::init(dir).unwrap();
}