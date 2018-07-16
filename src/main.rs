#[macro_use]
extern crate quicli;
extern crate dialoguer;
extern crate git2;
extern crate ident_case;
extern crate indicatif;
extern crate liquid;
extern crate regex;
extern crate walkdir;

mod cargo;
mod interactive;
mod git;
mod progressbar;
mod template;

use quicli::prelude::*;
use std::env;

/// Generate a new Cargo project from a given template
///
/// Right now, only git repositories can be used as templates. Just execute
///
/// $ cargo generate --git https://github.com/user/template.git --name foo
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
#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(long = "git")]
    git: String,
    #[structopt(long = "name")]
    name: Option<String>,
}

main!(|args: Cli| {
    let name = match &args.name {
        Some(ref n) => n.to_string(),
        None => interactive::name()?,
    };

    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&name);

    ensure!(
        !project_dir.exists(),
        "Target directory `{}` already exists, aborting.",
        project_dir.display()
    );

    git::create(&project_dir, args)?;
    git::remove_history(&project_dir)?;

    let mut template = template::new();
    template = template::substitute(&name, template)?;

    let pbar = progressbar::new();
    pbar.tick();

    template::walk_dir(&project_dir, template, pbar)?;

    git::init(&project_dir)?;

    println!("Done!");
});
