use anyhow::{Context, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar};
use liquid::Parser;
use liquid_core::{Object, Value};
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::config::TemplateConfig;
use crate::emoji;
use crate::filenames::substitute_filename;
use crate::include_exclude::*;
use crate::progressbar::spinner;
use crate::template_filters::{
    KebabCaseFilterParser, PascalCaseFilterParser, SnakeCaseFilterParser,
};
use crate::template_variables::{get_authors, get_os_arch, Authors, CrateType, ProjectName};

fn engine() -> liquid::Parser {
    liquid::ParserBuilder::with_stdlib()
        .filter(KebabCaseFilterParser)
        .filter(PascalCaseFilterParser)
        .filter(SnakeCaseFilterParser)
        .build()
        .expect("can't fail due to no partials support")
}

pub fn create_liquid_object(
    name: &ProjectName,
    crate_type: &CrateType,
    force: bool,
) -> Result<Object> {
    let authors: Authors = get_authors()?;
    let os_arch = get_os_arch();
    let project_name = if force { name.raw() } else { name.kebab_case() };

    let mut liquid_object = Object::new();
    liquid_object.insert("project-name".into(), Value::Scalar(project_name.into()));
    liquid_object.insert("crate_name".into(), Value::Scalar(name.snake_case().into()));
    liquid_object.insert(
        "crate_type".into(),
        Value::Scalar(crate_type.to_string().into()),
    );
    liquid_object.insert("authors".into(), Value::Scalar(authors.into()));
    liquid_object.insert("os-arch".into(), Value::Scalar(os_arch.into()));

    Ok(liquid_object)
}

pub fn walk_dir(
    project_dir: &Path,
    liquid_object: &Object,
    template_config: &mut TemplateConfig,
    hook_files: &[String],
    mp: &mut MultiProgress,
) -> Result<()> {
    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .components()
            .any(|c| c == std::path::Component::Normal(".git".as_ref()))
    }

    let engine = engine();

    let matcher = Matcher::new(template_config, project_dir, hook_files)?;
    let spinner_style = spinner();

    let mut files_with_errors = Vec::new();
    let files = WalkDir::new(project_dir)
        .sort_by_file_name()
        .contents_first(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !is_git_metadata(e))
        .filter(|e| e.path() != project_dir)
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

        // todo(refactor): as parameter
        let verbose = false;

        match matcher.should_include(relative_path) {
            ShouldInclude::Include => {
                if entry.file_type().is_file() {
                    match template_process_file(liquid_object, &engine, filename) {
                        Err(e) => {
                            if verbose {
                                files_with_errors.push((filename.display().to_string(), e.clone()));
                            }
                        }
                        Ok(new_contents) => {
                            let new_filename =
                                substitute_filename(filename, &engine, liquid_object)
                                    .with_context(|| {
                                        format!(
                                            "{} {} `{}`",
                                            emoji::ERROR,
                                            style("Error templating a filename").bold().red(),
                                            style(filename.display()).bold()
                                        )
                                    })?;
                            pb.inc(25);
                            let relative_path = new_filename.strip_prefix(project_dir)?;
                            let f = relative_path.display();
                            fs::create_dir_all(new_filename.parent().unwrap()).unwrap();
                            fs::write(new_filename.as_path(), new_contents).with_context(|| {
                                format!(
                                    "{} {} `{}`",
                                    emoji::ERROR,
                                    style("Error writing rendered file.").bold().red(),
                                    style(new_filename.display()).bold()
                                )
                            })?;
                            pb.inc(50);
                            pb.finish_with_message(format!("Done: {}", f));
                        }
                    }
                } else {
                    let new_filename = substitute_filename(filename, &engine, liquid_object)?;
                    let relative_path = new_filename.strip_prefix(project_dir)?;
                    let f = relative_path.display();
                    pb.inc(50);
                    if filename != new_filename {
                        fs::remove_dir_all(filename)?;
                    }
                    pb.inc(50);
                    pb.finish_with_message(format!("Done: {}", f));
                }
            }
            ShouldInclude::Exclude => {
                pb.finish_with_message(format!("Skipped: {}", f));
            }
            ShouldInclude::Ignore => {
                pb.finish_with_message(format!("Ignored: {}", f));
            }
        }
    }

    if !files_with_errors.is_empty() {
        print_files_with_errors_warning(files_with_errors);
    }

    Ok(())
}

fn template_process_file(
    context: &Object,
    parser: &Parser,
    file: &Path,
) -> liquid_core::Result<String> {
    let content =
        fs::read_to_string(file).map_err(|e| liquid_core::Error::with_msg(e.to_string()))?;
    render_string_gracefully(context, parser, content.as_str())
}

pub fn render_string_gracefully(
    context: &Object,
    parser: &Parser,
    content: &str,
) -> liquid_core::Result<String> {
    let template = parser.parse(content)?;

    match template.render(context) {
        Ok(content) => Ok(content),
        Err(e) => {
            // handle it gracefully
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
                    return render_string_gracefully(&context, parser, content);
                }
            }
            // todo: find nice way to have this happening outside of this fn
            // println!(
            //     "{} {} `{}`",
            //     emoji::ERROR,
            //     style("Error rendering template, file has been copied without rendering.")
            //         .bold()
            //         .red(),
            //     style(filename.display()).bold()
            // );
            // todo: end

            // fallback: no rendering, keep things original
            Ok(content.to_string())
        }
    }
}

fn print_files_with_errors_warning(files_with_errors: Vec<(String, liquid_core::Error)>) {
    let mut msg = format!(
        "\n{} {}",
        emoji::WARN,
        style("Substitution skipped, found invalid syntax in\n")
            .bold()
            .red(),
    );
    for file_error in files_with_errors {
        msg.push('\t');
        msg.push_str(&file_error.0);
        msg.push('\n');
    }
    let read_more =
        "Learn more: https://github.com/cargo-generate/cargo-generate#include--exclude.\n\n";
    let hint = style("Consider adding these files to a `cargo-generate.toml` in the template repo to skip substitution on these files.").bold();

    println!("{}\n{}\n\n{}", msg, hint, read_more);
}
