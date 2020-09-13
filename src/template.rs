use crate::authors;
use crate::config::TemplateConfig;
use crate::emoji;
use crate::include_exclude::*;
use crate::projectname::ProjectName;
use anyhow::{Context, Result};
use console::style;
use heck::{CamelCase, KebabCase, SnakeCase};
use indicatif::ProgressBar;
use liquid_core::{Filter, FilterReflection, Object, ParseFilter, Runtime, ValueView, Value};
use std::env;
use liquid::Template;
use liquid_core::{Filter, FilterReflection, Object, ParseFilter, Runtime, Value, ValueView};
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
    let os_arch = format!("{}-{}", env::consts::OS, env::consts::ARCH);

    Ok(liquid::object!({
        "project-name": project_name,
        "crate_name": name.snake_case(),
        "authors": authors,
        "os-arch": os_arch,
    }))
}

pub(crate) fn walk_dir(
    project_dir: &Path,
    template: Object,
    template_config: Option<TemplateConfig>,
    pbar: ProgressBar,
    verbose: bool,
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

    let mut files_with_errors = Vec::new();
    for entry in WalkDir::new(project_dir) {
        let entry = entry?;
        if is_dir(&entry) || is_git_metadata(&entry) {
            continue;
        }

        let filename = entry.path();
        let relative_path = filename.strip_prefix(project_dir)?;
        pbar.set_message(&filename.display().to_string());

        if matcher.should_include(relative_path) {
            let parsed_file = engine.clone().parse_file(filename);
            match parsed_file {
                Ok(parsed_file) => parse_and_replace(&template, filename, parsed_file)?,
                Err(e) => {
                    if verbose {
                        files_with_errors.push((filename.display().to_string(), e.clone()));
                    }
                }
            }
        }
    }
    if files_with_errors.len() > 0 {
        let warn = construct_substitution_warning(files_with_errors);
        println!("{}", warn);
    }

    pbar.finish_and_clear();
    Ok(())
}

fn parse_and_replace(
    context: &Object,
    filename: &Path,
    parsed_file: Template,
) -> Result<(), anyhow::Error> {
    let new_content = parse_file(context, filename, &parsed_file);
    fs::write(filename, new_content).with_context(|| {
        format!(
            "{} {} `{}`",
            emoji::ERROR,
            style("Error writing").bold().red(),
            style(filename.display()).bold()
        )
    })?;

    Ok(())
}

fn parse_file(context: &Object, filename: &Path, parsed_file: &Template) -> String {
    match parsed_file.render(context) {
        Ok(content) => content,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("requested variable") {
                // so, we miss a variable that is present in the file to render
                let requested_var =
                    regex::Regex::new(r"(?P<p>.*requested\svariable=)(?P<v>.*)").unwrap();
                let captures = requested_var.captures(msg.as_str()).unwrap();

                if let Some(Some(req_var)) = captures.iter().last() {
                    let missing_variable = req_var.as_str().to_string();
                    // try again with this variable added to the context
                    let mut context = context.clone();
                    context.insert(missing_variable.into(), Value::scalar("".to_string()));
                    // now let's parse again to see if we have all variables declared now
                    return parse_file(&context, filename, parsed_file);
                }
            }
            eprintln!(
                "{} {} `{}`",
                emoji::ERROR,
                style("Error replacing placeholders, file got only copied.")
                    .bold()
                    .red(),
                style(filename.display()).bold()
            );
            // fallback: take the file as it is, without substitutions
            fs::read_to_string(filename).unwrap()
        }
    }
}

fn construct_substitution_warning(files_with_errors: Vec<(String, liquid_core::Error)>) -> String {
    let mut msg = format!(
        "\n{} {}",
        emoji::WARN,
        style("Substitution skipped, found invalid syntax in\n")
            .bold()
            .red(),
    );
    for file_error in files_with_errors {
        msg.push_str("\t");
        msg.push_str(&file_error.0);
        msg.push_str("\n");
    }
    msg.push_str("\n");
    let info = format!("{}", style("Consider adding these files to a `cargo-generate.toml` in the template repo to skip substitution on these files.\n").bold());
    msg.push_str(&info);
    msg.push_str(
        "Learn more: https://github.com/ashleygwilliams/cargo-generate#include--exclude.\n\n",
    );
    msg
}
