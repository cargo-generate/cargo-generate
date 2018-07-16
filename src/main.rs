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

use quicli::prelude::*;
use std::{env, fs};
use walkdir::WalkDir;

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

    let engine = liquid::ParserBuilder::new().build();
    let mut placeholders = liquid::Object::new();
    placeholders.insert(String::from("project-name"), liquid::Value::scalar(&name));
    placeholders.insert(
        String::from("crate_name"),
        liquid::Value::scalar(&ident_case::RenameRule::SnakeCase.apply_to_field(&name)),
    );
    placeholders.insert(
        String::from("authors"),
        liquid::Value::scalar(&cargo::get_authors()?),
    );

    let progress = indicatif::ProgressBar::new_spinner();
    progress.tick();

    for entry in WalkDir::new(&project_dir) {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            continue;
        }

        let filename = entry.path();
        progress.set_message(&filename.display().to_string());

        let new_contents = engine
            .clone()
            .parse_file(&filename)?
            .render(&placeholders)
            .with_context(|_e| {
                format!("Error replacing placeholders in `{}`", filename.display())
            })?;
        fs::write(&filename, new_contents)
            .with_context(|_e| format!("Error writing `{}`", filename.display()))?;
    }

    git::init(&project_dir)?;

    progress.finish_and_clear();
    println!("Done!");
});
