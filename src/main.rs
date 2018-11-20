#[macro_use]
extern crate quicli;
extern crate cargo as upstream;
extern crate console;
extern crate dialoguer;
extern crate git2;
extern crate heck;
extern crate ignore;
extern crate indicatif;
extern crate liquid;
extern crate regex;
extern crate remove_dir_all;
extern crate tempfile;
extern crate url;
extern crate walkdir;

mod cargo;
mod emoji;
mod git;
mod ignoreme;
mod interactive;
mod progressbar;
mod projectname;
mod template;

use console::style;
use git::GitConfig;
use projectname::ProjectName;
use quicli::prelude::*;
use std::env;

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

main!(|_cli: Cli| {
    let args: Args = match Cli::from_args() {
        Cli::Generate(args) => args,
        Cli::Gen(args) => args,
    };

    let name = match &args.name {
        Some(ref n) => ProjectName::new(n),
        None => ProjectName::new(&interactive::name()?),
    };
    let force = args.force;
    let config = GitConfig::new(args.git, args.branch)?;

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

    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Creating project called").bold(),
        style(&name.kebab_case()).bold().yellow(),
        style("...").bold()
    );

    let dir_name = if force { name.raw() } else { name.kebab_case() };
    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(dir_name);

    ensure!(
        !project_dir.exists(),
        "Target directory `{}` already exists, aborting.",
        project_dir.display()
    );

    git::create(&project_dir, config)?;

    let template = template::substitute(&name, force)?;

    let pbar = progressbar::new();
    pbar.tick();

    template::walk_dir(&project_dir, template, pbar)?;

    git::remove_history(&project_dir)?;
    git::init(&project_dir)?;

    //remove uneeded here
    ignoreme::remove_uneeded_files(&project_dir);

    let dir_string = &project_dir.to_str().unwrap_or("");
    println!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(dir_string).underlined()
    );
});
