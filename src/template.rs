use anyhow::{Context, Result};
use console::style;
use heck::{CamelCase, KebabCase, SnakeCase};
use indicatif::{MultiProgress, ProgressBar};
use liquid_core::{Filter, Object, ParseFilter, Runtime, Value, ValueView};
use liquid_derive::FilterReflection;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::config::TemplateConfig;
use crate::emoji;
use crate::include_exclude::*;
use crate::progressbar::spinner;
use crate::template_variables::{get_authors, get_os_arch, Authors, CrateType, ProjectName};

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
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_string().to_kebab_case();
        Ok(liquid_core::model::Value::scalar(input))
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
        _runtime: &dyn Runtime,
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
        _runtime: &dyn Runtime,
    ) -> Result<liquid::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_snake_case();
        Ok(input.to_value())
    }
}

pub(crate) fn substitute(
    name: &ProjectName,
    crate_type: &CrateType,
    template_values: &HashMap<String, toml::Value>,
    force: bool,
) -> Result<Object> {
    let project_name = if force { name.raw() } else { name.kebab_case() };
    let authors: Authors = get_authors()?;
    let os_arch = get_os_arch();

    let mut liquid_object = Object::new();
    liquid_object.insert("project-name".into(), Value::Scalar(project_name.into()));
    liquid_object.insert("crate_name".into(), Value::Scalar(name.snake_case().into()));
    liquid_object.insert(
        "crate_type".into(),
        Value::Scalar(crate_type.to_string().into()),
    );
    liquid_object.insert("authors".into(), Value::Scalar(authors.into()));
    liquid_object.insert("os-arch".into(), Value::Scalar(os_arch.into()));

    template_values.iter().try_for_each(|(k, v)| {
        let value = match v {
            toml::Value::String(content) => Value::Scalar(content.clone().into()),
            toml::Value::Boolean(content) => Value::Scalar((*content).into()),
            _ => anyhow::bail!(format!(
                "{} {}",
                emoji::ERROR,
                style("Unsupported value type. Only Strings and Booleans are supported.")
                    .bold()
                    .red(),
            )),
        };
        liquid_object.insert(k.clone().into(), value);
        Ok(())
    })?;

    Ok(liquid_object)
}

pub(crate) fn walk_dir(
    project_dir: &Path,
    template: Object,
    template_config: Option<TemplateConfig>,
    mp: &mut MultiProgress,
) -> Result<()> {
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
    let spinner_style = spinner();

    let files = WalkDir::new(project_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| !is_git_metadata(e))
        .collect::<Vec<_>>();
    let total = files.len().to_string();
    for (progress, entry) in files.into_iter().enumerate() {
        let pb = mp.add(ProgressBar::new(50));
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!(
            "[{:width$}/{}]",
            progress + 1,
            total,
            width = total.len()
        ));

        let filename = entry.path();
        let relative_path = filename.strip_prefix(project_dir)?;
        let f = relative_path.display();
        pb.set_message(format!("Processing: {}", f));

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
            pb.inc(50);
            fs::write(filename, new_contents).with_context(|| {
                format!(
                    "{} {} `{}`",
                    emoji::ERROR,
                    style("Error writing").bold().red(),
                    style(filename.display()).bold()
                )
            })?;
            pb.inc(50);
            pb.finish_with_message(format!("Done: {}", f));
        } else {
            pb.finish_with_message(format!("Skipped: {}", f));
        }
    }

    Ok(())
}
