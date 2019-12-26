mod authors;
mod config;
mod emoji;
mod git;
mod ignoreme;
mod include_exclude;
mod interactive;
mod progressbar;
mod projectname;
mod template;

use crate::git::GitConfig;
use crate::projectname::ProjectName;
use cargo;
use config::{Config, CONFIG_FILE_NAME};
use console::style;
use failure;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(long = "verbose", short = "v")]
    verbose: bool,
}

macro_rules! hashmap {
    ($($a:expr => $b:expr,)*) => {
        vec!($( ($a.into(), $b.into()) ),*).into_iter().collect()
    };
}

thread_local! {
    static GIT_MAP: HashMap<String, String> = {
        hashmap!(
            "py03" => "https://github.com/DD5HT/pyo3-template",
            "wasm-pack" => "https://github.com/rustwasm/wasm-pack-template",
            "cli" => "https://github.com/rust-cli/cli-template",
            "rocket-base" => "https://github.com/k0pernicus/cargo-template-rocket-base",
            "rust-samp-sdk" => "https://github.com/Sreyas-Sreelal/rs-plugin-boilerplate",
            "actix-tera" => "https://github.com/otomato-gh/cargo-template-actix-tera",
            "procmacro-quickstart" => "https://github.com/eupn/rust-procmacro-quickstart-template",
            "bluepill" => "https://github.com/mendelt/bluepill-template",
            "cmdr" => "https://github.com/mendelt/cmdr-template",
            "ggez" => "https://github.com/cyclowns/cargo-generate-ggez",
            "generust" => "https://github.com/kyleu/generust",
        )
    }
}

pub fn generate(args: Args) -> Result<(), failure::Error> {
    let name = match &args.name {
        Some(ref n) => ProjectName::new(n),
        None => ProjectName::new(&interactive::name()?),
    };

    create_git(args, &name)?;

    Ok(())
}

fn create_git(mut args: Args, name: &ProjectName) -> Result<(), failure::Error> {
    match_git_map(&mut args);

    let force = args.force;
    let branch = args.branch.unwrap_or_else(|| "master".to_string());
    let config = GitConfig::new(args.git, branch.clone())?;
    let verbose = args.verbose;
    if let Some(dir) = &create_project_dir(&name, force) {
        match git::create(dir, config) {
            Ok(_) => {
                git::remove_history(dir).unwrap_or(progress(name, dir, force, &branch, verbose)?)
            }
            Err(e) => failure::bail!(
                "{} {} {}",
                emoji::ERROR,
                style("Git Error:").bold().red(),
                style(e).bold().red(),
            ),
        };
    } else {
        failure::bail!(
            "{} {}",
            emoji::ERROR,
            style("Target directory already exists, aborting!")
                .bold()
                .red(),
        );
    }
    Ok(())
}

/// Maps a short name to the git url, includes all templates listed in TEMPLATES.md.
fn match_git_map(args: &mut Args) {
    GIT_MAP.with(|map| {
        if let Some(url) = map.get(&args.git) {
            args.git = url.clone();
        }
    })
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
    dir: &PathBuf,
    force: bool,
    branch: &str,
    verbose: bool,
) -> Result<(), failure::Error> {
    let template = template::substitute(name, force)?;

    let mut config_path = dir.clone();
    config_path.push(CONFIG_FILE_NAME);

    let template_config = Config::new(config_path)?.map(|c| c.template);

    let pbar = progressbar::new();
    pbar.tick();

    template::walk_dir(dir, template, template_config, pbar)?;

    git::init(dir, branch)?;

    ignoreme::remove_unneeded_files(dir, verbose);

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
