use crate::authors;
use crate::emoji;
use crate::projectname::ProjectName;
use console::style;
use failure;
use heck::{CamelCase, KebabCase, SnakeCase};
use indicatif::ProgressBar;
use liquid;
use quicli::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn engine() -> liquid::Parser {
    liquid::ParserBuilder::new()
        .filter(liquid::filters::std::Date)
        .filter(liquid::filters::std::Capitalize)
        .filter(KebabCaseFilterParser)
        .filter(PascalCaseFilterParser)
        .filter(SnakeCaseFilterParser)
        .build()
        .expect("can't fail due to no partials support")
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "kebab_case",
    description = "Change text to kebab-case.",
    parsed(KebabCaseFilter)
)]
pub struct KebabCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "kebab_case"]
struct KebabCaseFilter;

impl liquid::compiler::Filter for KebabCaseFilter {
    fn evaluate(
        &self,
        input: &liquid::value::Value,
        context: &liquid::interpreter::Context,
    ) -> Result<liquid::value::Value, liquid::error::Error> {
        let input = input.to_str();
        let input = input.as_ref().to_kebab_case();
        Ok(liquid::value::Value::scalar(input))
    }
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "kebab_case",
    description = "Change text to PascalCase.",
    parsed(PascalCaseFilter)
)]
pub struct PascalCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "pascal_case"]
struct PascalCaseFilter;

impl liquid::compiler::Filter for PascalCaseFilter {
    fn evaluate(
        &self,
        input: &liquid::value::Value,
        context: &liquid::interpreter::Context,
    ) -> Result<liquid::value::Value, liquid::error::Error> {
        let input = input.to_str();
        let input = input.as_ref().to_camel_case();
        Ok(liquid::value::Value::scalar(input))
    }
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "kebab_case",
    description = "Change text to snake_case.",
    parsed(SnakeCaseFilter)
)]
pub struct SnakeCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "pascal_case"]
struct SnakeCaseFilter;

impl liquid::compiler::Filter for SnakeCaseFilter {
    fn evaluate(
        &self,
        input: &liquid::value::Value,
        context: &liquid::interpreter::Context,
    ) -> Result<liquid::value::Value, liquid::error::Error> {
        let input = input.to_str();
        let input = input.as_ref().to_snake_case();
        Ok(liquid::value::Value::scalar(input))
    }
}

pub fn substitute(
    name: &ProjectName,
    force: bool,
) -> Result<liquid::value::Object, failure::Error> {
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
        liquid::value::Value::scalar(authors::get_authors()?),
    );
    Ok(template)
}

pub fn walk_dir(
    project_dir: &PathBuf,
    template: liquid::value::Object,
    pbar: ProgressBar,
) -> Result<(), failure::Error> {
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

        let filepath = entry.path();
        pbar.set_message(&filepath.display().to_string());

        // Override the content
        let new_contents = engine
            .clone()
            .parse_file(&filepath)?
            .render(&template)
            .with_context(|_e| {
                format!(
                    "{} {} `{}`",
                    emoji::ERROR,
                    style("Error replacing placeholders").bold().red(),
                    style(filepath.display()).bold()
                )
            })?;
        fs::write(&filepath, new_contents).with_context(|_e| {
            format!(
                "{} {} `{}`",
                emoji::ERROR,
                style("Error writing").bold().red(),
                style(filepath.display()).bold()
            )
        })?;

        // Check if the filename does not contains any
        // template
        let filename = filepath.to_str().unwrap();
        let parsed_filename = engine.clone().parse(filename)?.render(&template)?;
        let _ = fs::rename(&filepath, Path::new(&parsed_filename)).with_context(|_e| {
            format!(
                "{} {} '{}'",
                emoji::ERROR,
                style("Error renaming").bold().red(),
                style(parsed_filename).bold()
            )
        })?;
    }
    pbar.finish_and_clear();
    Ok(())
}
