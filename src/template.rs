use cargo;
use console::style;
use emoji;
use indicatif::ProgressBar;
use liquid;
use projectname::ProjectName;
use quicli::prelude::*;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn engine() -> liquid::Parser {
    liquid::ParserBuilder::new().build()
}

pub fn substitute(name: &ProjectName, force: bool) -> Result<liquid::Object> {
    let project_name =
        if force { name.raw() } else { name.kebab_case() };

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
    let engine = engine();
    for entry in WalkDir::new(project_dir) {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
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
