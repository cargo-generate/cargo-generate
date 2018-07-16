use cargo;
use ident_case;
use indicatif::ProgressBar;
use liquid;
use quicli::prelude::*;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn engine() -> liquid::Parser {
    liquid::ParserBuilder::new().build()
}

pub fn substitute(name: &str) -> Result<liquid::Object> {
    let mut template = liquid::Object::new();
    template.insert(String::from("project-name"), liquid::Value::scalar(name));
    template.insert(
        String::from("crate_name"),
        liquid::Value::scalar(&ident_case::RenameRule::SnakeCase.apply_to_field(name)),
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
                format!("Error replacing placeholders in `{}`", filename.display())
            })?;
        fs::write(&filename, new_contents)
            .with_context(|_e| format!("Error writing `{}`", filename.display()))?;
    }
    pbar.finish_and_clear();
    Ok(())
}
