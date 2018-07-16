#[macro_use]
extern crate quicli;
extern crate dialoguer;
extern crate git2;
extern crate ident_case;
extern crate indicatif;
extern crate liquid;
extern crate regex;
extern crate remove_dir_all;
extern crate walkdir;

mod cargo;

use dialoguer::Input;
use git2::{build::CheckoutBuilder, build::RepoBuilder, Repository as GitRepository,
           RepositoryInitOptions};
use quicli::prelude::*;
use remove_dir_all::remove_dir_all;
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
struct Cli {
    #[structopt(long = "git")]
    git: String,
    #[structopt(long = "name")]
    name: Option<String>,
}

main!(|args: Cli| {
    let name = match &args.name {
        Some(ref n) => n.to_string(),
        None => query_name()?,
    };

    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&name);

    ensure!(
        !project_dir.exists(),
        "Target directory `{}` already exists, aborting.",
        project_dir.display()
    );

    let _template = RepoBuilder::new()
        .bare(false)
        .with_checkout(CheckoutBuilder::new())
        .clone(&args.git, &project_dir)
        .with_context(|_e| format!("Couldn't clone `{}`", &args.git))?;

    remove_dir_all(&project_dir.join(".git")).context("Error cleaning up cloned template")?;

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

    let _repo = GitRepository::init_opts(&project_dir, RepositoryInitOptions::new().bare(false))
        .context("Couldn't init new repository")?;

    progress.finish_and_clear();
    println!("Done!");
});

fn query_name() -> Result<String> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let name = loop {
        let name = Input::new("The project's name is").interact()?;
        if valid_ident.is_match(&name) {
            let name = ident_case::RenameRule::KebabCase.apply_to_field(&name);
            println!("Nice, I'll call your project `{}`", name);
            break name;
        } else {
            eprintln!("Sorry, that is not a valid crate name :(");
        }
    };
    Ok(name)
}
