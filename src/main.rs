#[macro_use]
extern crate quicli;
extern crate dialoguer;
extern crate git2;
extern crate ident_case;
extern crate indicatif;
extern crate liquid;
extern crate regex;
extern crate walkdir;

use dialoguer::Input;
use git2::{Config as GitConfig, Repository as GitRepository, RepositoryInitOptions};
use quicli::prelude::*;
use std::{env, fs};
use walkdir::WalkDir;

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

    let _template = GitRepository::clone(&args.git, &project_dir)
        .with_context(|_e| format!("Couldn't clone `{}`", &args.git))?;

    fs::remove_dir_all(&project_dir.join(".git")).context("Error cleaning up cloned template")?;

    let engine = liquid::ParserBuilder::new().build();
    let mut placeholders = liquid::Object::new();
    placeholders.insert(String::from("project-name"), liquid::Value::scalar(&name));
    placeholders.insert(
        String::from("crate_name"),
        liquid::Value::scalar(&ident_case::RenameRule::SnakeCase.apply_to_field(&name)),
    );
    placeholders.insert(
        String::from("authors"),
        liquid::Value::scalar(&get_authors()?),
    );

    let progress = indicatif::ProgressBar::new_spinner();
    progress.tick();

    for entry in WalkDir::new(&project_dir)
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with(".git"))
                .unwrap_or(false)
        }) {
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

/// Taken from cargo ans thus (c) 2018 Cargo Developers
///
/// cf. https://github.com/rust-lang/cargo/blob/d33c65cbd9d6f7ba1e18b2cdb85fea5a09973d3b/src/cargo/ops/cargo_new.rs#L595-L645
fn get_authors() -> Result<String> {
    fn get_environment_variable(variables: &[&str]) -> Option<String> {
        variables.iter().filter_map(|var| env::var(var).ok()).next()
    }

    fn discover_author() -> Result<(String, Option<String>)> {
        let cwd = env::current_dir()?;
        let git_config = if let Ok(repo) = GitRepository::discover(&cwd) {
            repo.config()
                .ok()
                .or_else(|| GitConfig::open_default().ok())
        } else {
            GitConfig::open_default().ok()
        };
        let git_config = git_config.as_ref();
        let name_variables = [
            "CARGO_NAME",
            "GIT_AUTHOR_NAME",
            "GIT_COMMITTER_NAME",
            "USER",
            "USERNAME",
            "NAME",
        ];
        let name = get_environment_variable(&name_variables[0..3])
            .or_else(|| git_config.and_then(|g| g.get_string("user.name").ok()))
            .or_else(|| get_environment_variable(&name_variables[3..]));

        let name = match name {
            Some(name) => name,
            None => {
                let username_var = if cfg!(windows) { "USERNAME" } else { "USER" };
                bail!(
                    "could not determine the current user, please set ${}",
                    username_var
                )
            }
        };
        let email_variables = [
            "CARGO_EMAIL",
            "GIT_AUTHOR_EMAIL",
            "GIT_COMMITTER_EMAIL",
            "EMAIL",
        ];
        let email = get_environment_variable(&email_variables[0..3])
            .or_else(|| git_config.and_then(|g| g.get_string("user.email").ok()))
            .or_else(|| get_environment_variable(&email_variables[3..]));

        let name = name.trim().to_string();
        let email = email.map(|s| s.trim().to_string());

        Ok((name, email))
    }

    let author = match discover_author()? {
        (name, Some(email)) => format!("{} <{}>", name, email),
        (name, None) => name,
    };

    Ok(author)
}
