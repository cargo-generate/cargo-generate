use crate::authors;
use crate::config::TemplateConfig;
use crate::emoji;
use crate::include_exclude::*;
use crate::projectname::ProjectName;
use anyhow::{Context, Result};
use console::style;
use heck::{CamelCase, KebabCase, SnakeCase};
use indicatif::ProgressBar;
use liquid_core::{Filter, FilterReflection, Object, ParseFilter, Runtime, ValueView};
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn engine() -> liquid::Parser {
    liquid::ParserBuilder::with_stdlib()
        .filter(KebabCaseFilterParser)
        .filter(PascalCaseFilterParser)
        .filter(SnakeCaseFilterParser)
        .build()
        .expect("can't fail due to no partials support")
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "kebab_case",
    description = "Change text to kebab-case.",
    parsed(KebabCaseFilter)
)]
pub(crate) struct KebabCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "kebab_case"]
struct KebabCaseFilter;

impl Filter for KebabCaseFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &Runtime,
    ) -> Result<liquid::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_string().to_kebab_case();
        Ok(liquid::model::Value::scalar(input))
    }
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "pascal_case",
    description = "Change text to PascalCase.",
    parsed(PascalCaseFilter)
)]
pub(crate) struct PascalCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "pascal_case"]
struct PascalCaseFilter;

impl Filter for PascalCaseFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &Runtime,
    ) -> Result<liquid::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_camel_case();
        Ok(liquid::model::Value::scalar(input))
    }
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "snake_case",
    description = "Change text to snake_case.",
    parsed(SnakeCaseFilter)
)]
pub(crate) struct SnakeCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "snake_case"]
struct SnakeCaseFilter;

impl Filter for SnakeCaseFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &Runtime<'_>,
    ) -> Result<liquid::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_snake_case();
        Ok(input.to_value())
    }
}

pub(crate) fn substitute(name: &ProjectName, force: bool) -> Result<Object> {
    let project_name = if force { name.raw() } else { name.kebab_case() };
    let authors = authors::get_authors()?;

    Ok(liquid::object!({
        "project-name": project_name,
        "crate_name": name.snake_case(),
        "authors": authors,
    }))
}

pub(crate) fn walk_dir(
    project_dir: &Path,
    template: Object,
    template_config: Option<TemplateConfig>,
    pbar: ProgressBar,
) -> Result<()> {
    fn is_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
    }

    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .components()
            .any(|c| c == std::path::Component::Normal(".git".as_ref()))
    }

    let engine = engine();

    let matcher = template_config.map_or_else(
        || Ok(Matcher::default()),
        |config| Matcher::new(config, project_dir),
    )?;

    for entry in WalkDir::new(project_dir) {
        let entry = entry?;
        if is_dir(&entry) || is_git_metadata(&entry) {
            continue;
        }

        let filename = entry.path();
        let relative_path = filename.strip_prefix(project_dir)?;
        pbar.set_message(&filename.display().to_string());

        if matcher.should_include(relative_path) {
            let new_contents = engine
                .clone()
                .parse_file(filename)?
                .render(&template)
                .with_context(|| {
                    format!(
                        "{} {} `{}`",
                        emoji::ERROR,
                        style("Error replacing placeholders").bold().red(),
                        style(filename.display()).bold()
                    )
                })?;
            fs::write(filename, new_contents).with_context(|| {
                format!(
                    "{} {} `{}`",
                    emoji::ERROR,
                    style("Error writing").bold().red(),
                    style(filename.display()).bold()
                )
            })?;
        }
    }

    pbar.finish_and_clear();
    Ok(())
}
