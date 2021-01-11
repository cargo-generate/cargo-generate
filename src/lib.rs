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
use anyhow::Result;
use config::{Config, CONFIG_FILE_NAME};
use console::style;
use std::env;
use std::path::{Path, PathBuf};
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
    #[structopt(name = "generate", visible_alias = "gen")]
    Generate(Args),
}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[structopt(short, long)]
    pub git: String,
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
}

pub fn generate(args: Args) -> Result<()> {
    let name = match args.name {
        Some(ref n) => ProjectName::new(n),
        None => ProjectName::new(interactive::name()?),
    };

    create_git(args, &name)?;

    Ok(())
}

fn create_git(args: Args, name: &ProjectName) -> Result<()> {
    let force = args.force;
    let config = GitConfig::new_abbr(&args.git, args.branch.to_owned())?;
    let verbose = args.verbose;
    if let Some(dir) = &create_project_dir(&name, force) {
        match git::create(dir, config) {
            Ok(branch) => {
                git::remove_history(dir)?;
                progress(name, dir, force, &branch, verbose)?;
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
    dir: &Path,
    force: bool,
    branch: &str,
    verbose: bool,
) -> Result<()> {
    let template = template::substitute(name, force)?;

    let config_path = dir.join(CONFIG_FILE_NAME);

    let template_config = Config::new(config_path)?.map(|c| c.template);

    let pbar = progressbar::new();
    pbar.tick();

    ignoreme::remove_unneeded_files(dir, verbose);

    template::walk_dir(dir, template, template_config, pbar)?;

    git::init(dir, branch)?;

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
