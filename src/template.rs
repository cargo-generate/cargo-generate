use cargo;
use console::style;
use emoji;
use heck::{CamelCase, KebabCase, SnakeCase};
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
            liquid::filters::date as liquid::compiler::FnFilterValue,
        )
        .filter(
            "capitalize",
            liquid::filters::capitalize as liquid::compiler::FnFilterValue,
        )
        .filter("kebab_case", kebab_case as liquid::compiler::FnFilterValue)
        .filter(
            "pascal_case",
            pascal_case as liquid::compiler::FnFilterValue,
        )
        .filter("snake_case", snake_case as liquid::compiler::FnFilterValue)
        .build()
        .expect("can't fail due to no partials support")
}

fn kebab_case(
    input: &liquid::value::Value,
    _args: &[liquid::value::Value],
) -> ::std::result::Result<liquid::value::Value, liquid::Error> {
    let input = input.to_str();
    let input = input.as_ref().to_kebab_case();
    Ok(liquid::value::Value::scalar(input))
}

fn pascal_case(
    input: &liquid::value::Value,
    _args: &[liquid::value::Value],
) -> ::std::result::Result<liquid::value::Value, liquid::Error> {
    let input = input.to_str();
    let input = input.as_ref().to_camel_case();
    Ok(liquid::value::Value::scalar(input))
}

fn snake_case(
    input: &liquid::value::Value,
    _args: &[liquid::value::Value],
) -> ::std::result::Result<liquid::value::Value, liquid::Error> {
    let input = input.to_str();
    let input = input.as_ref().to_snake_case();
    Ok(liquid::value::Value::scalar(input))
}

pub fn substitute(name: &ProjectName, force: bool) -> Result<liquid::value::Object> {
    let project_name = if force { name.raw() } else { name.kebab_case() };

    let mut template = liquid::value::Object::new();
    template.insert(
        "project-name".into(),
        liquid::value::Value::scalar(project_name),
    );
    template.insert(
        "crate_name".into(),
        liquid::value::Value::scalar(name.snake_case()),
    );
    template.insert(
        "authors".into(),
        liquid::value::Value::scalar(cargo::get_authors()?),
    );
    Ok(template)
}

pub fn walk_dir(
    project_dir: &PathBuf,
    template: liquid::value::Object,
    pbar: ProgressBar,
) -> Result<()> {
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
