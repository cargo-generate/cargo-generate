use cargo;
use console::style;
use emoji;
use indicatif::ProgressBar;
use liquid;
use projectname::ProjectName;
use quicli::prelude::*;
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

fn engine() -> liquid::Parser {
    liquid::ParserBuilder::new()
        .filter(
            "date",
            liquid::filters::date as liquid::interpreter::FnFilterValue,
        ).build()
}

pub fn substitute(name: &ProjectName, force: bool) -> Result<liquid::Object> {
    let project_name = if force { name.raw() } else { name.kebab_case() };

    let mut template = liquid::Object::new();
    template.insert(
        String::from("project-name"),
        liquid::Value::scalar(project_name),
    );
    template.insert(
        String::from("crate_name"),
        liquid::Value::scalar(&name.snake_case()),
    );
    template.insert(
        String::from("authors"),
        liquid::Value::scalar(&cargo::get_authors()?),
    );
    Ok(template)
}

pub fn walk_dir(project_dir: &PathBuf, template: liquid::Object, pbar: ProgressBar) -> Result<()> {
    fn is_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
    }

    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .to_str()
            .map(|s| s.contains(".git"))
            .unwrap_or(false)
    }

    let engine = engine();

    for entry in WalkDir::new(project_dir) {
        let entry = entry?;
        if is_dir(&entry) || is_git_metadata(&entry) {
            continue;
        }

        let filename = entry.path();
        pbar.set_message(&filename.display().to_string());

        let new_contents = engine
            .clone()
            .parse_file(&filename)?
            .render(&template)
            .with_context(|_e| {
                format!(
                    "{} {} `{}`",
                    emoji::ERROR,
                    style("Error replacing placeholders").bold().red(),
                    style(filename.display()).bold()
                )
            })?;
        fs::write(&filename, new_contents).with_context(|_e| {
            format!(
                "{} {} `{}`",
                emoji::ERROR,
                style("Error writing").bold().red(),
                style(filename.display()).bold()
            )
        })?;
    }
    pbar.finish_and_clear();
    Ok(())
}
