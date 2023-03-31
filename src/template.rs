use anyhow::{Context, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar};
use liquid::model::KString;
use liquid::{Parser, ParserBuilder};
use liquid_core::{Object, Value};
use log::warn;
use std::sync::{Arc, Mutex};
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

use crate::config::TemplateConfig;
use crate::emoji;
use crate::filenames::substitute_filename;
use crate::hooks::PoisonError;
use crate::include_exclude::*;
use crate::progressbar::spinner;
use crate::template_filters::*;
use crate::template_variables::{
    get_authors, get_os_arch, Authors, CrateName, ProjectDir, ProjectName,
};
use crate::user_parsed_input::UserParsedInput;

pub type LiquidObjectResource = Arc<Mutex<RefCell<Object>>>;

pub fn create_liquid_engine(
    template_dir: PathBuf,
    liquid_object: LiquidObjectResource,
    allow_commands: bool,
    silent: bool,
    rhai_filter_files: Arc<Mutex<Vec<PathBuf>>>,
) -> Parser {
    ParserBuilder::with_stdlib()
        .filter(KebabCaseFilterParser)
        .filter(LowerCamelCaseFilterParser)
        .filter(PascalCaseFilterParser)
        .filter(ShoutyKebabCaseFilterParser)
        .filter(ShoutySnakeCaseFilterParser)
        .filter(SnakeCaseFilterParser)
        .filter(TitleCaseFilterParser)
        .filter(UpperCamelCaseFilterParser)
        .filter(RhaiFilterParser::new(
            template_dir,
            liquid_object,
            allow_commands,
            silent,
            rhai_filter_files,
        ))
        .build()
        .expect("can't fail due to no partials support")
}

/// create liquid object for the template, and pre-fill it with all known variables
pub fn create_liquid_object(user_parsed_input: &UserParsedInput) -> Result<LiquidObjectResource> {
    let authors: Authors = get_authors()?;
    let os_arch = get_os_arch();

    let mut liquid_object = Object::new();

    if let Some(name) = user_parsed_input.name() {
        liquid_object.insert("project-name".into(), Value::Scalar(name.to_owned().into()));
    }

    liquid_object.insert(
        "crate_type".into(),
        Value::Scalar(user_parsed_input.crate_type().to_string().into()),
    );
    liquid_object.insert("authors".into(), Value::Scalar(authors.author.into()));
    liquid_object.insert("username".into(), Value::Scalar(authors.username.into()));
    liquid_object.insert("os-arch".into(), Value::Scalar(os_arch.into()));

    liquid_object.insert(
        "is_init".into(),
        Value::Scalar(user_parsed_input.init().into()),
    );

    Ok(Arc::new(Mutex::new(RefCell::new(liquid_object))))
}

pub fn set_project_name_variables(
    liquid_object: &LiquidObjectResource,
    project_dir: &ProjectDir,
    project_name: &ProjectName,
    crate_name: &CrateName,
) -> Result<()> {
    let ref_cell = liquid_object.lock().map_err(|_| PoisonError)?;
    let mut liquid_object = ref_cell.borrow_mut();

    liquid_object.insert(
        "project-name".into(),
        Value::Scalar(project_name.as_ref().to_owned().into()),
    );

    liquid_object.insert(
        "crate_name".into(),
        Value::Scalar(crate_name.as_ref().to_owned().into()),
    );

    liquid_object.insert(
        "within_cargo_project".into(),
        Value::Scalar(is_within_cargo_project(project_dir.as_ref()).into()),
    );

    Ok(())
}

fn is_within_cargo_project(project_dir: &Path) -> bool {
    Path::new(project_dir)
        .ancestors()
        .any(|folder| folder.join("Cargo.toml").exists())
}

#[allow(clippy::too_many_arguments)]
pub fn walk_dir(
    template_config: &mut TemplateConfig,
    project_dir: &Path,
    hook_files: &[String],
    liquid_object: &LiquidObjectResource,
    rhai_engine: Parser,
    rhai_filter_files: &Arc<Mutex<Vec<PathBuf>>>,
    mp: &mut MultiProgress,
    verbose: bool,
) -> Result<()> {
    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .components()
            .any(|c| c == std::path::Component::Normal(".git".as_ref()))
    }

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

        if !verbose {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }

        let filename = entry.path();
        let relative_path = filename.strip_prefix(project_dir)?;
        let filename_display = relative_path.display();
        // Attempt to NOT process files used as liquid rhai filters.
        // Only works if filter file has been used before an attempt to process it!
        if rhai_filter_files
            .lock()
            .map_err(|_| PoisonError)?
            .iter()
            .any(|rhai_filter| relative_path.eq(rhai_filter.as_path()))
        {
            pb.finish_with_message(format!(
                "Skipped: {filename_display} - used as Rhai filter!"
            ));
            continue;
        }

        pb.set_message(format!("Processing: {filename_display}"));

        match matcher.should_include(relative_path) {
            ShouldInclude::Include => {
                if entry.file_type().is_file() {
                    match template_process_file(liquid_object, &rhai_engine, filename) {
                        Err(e) => {
                            if verbose {
                                files_with_errors.push((filename.display().to_string(), e.clone()));
                            }
                        }
                        Ok(new_contents) => {
                            let new_filename =
                                substitute_filename(filename, &rhai_engine, liquid_object)
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
                            if filename != new_filename {
                                fs::remove_file(filename)?;
                            }
                            pb.inc(50);
                            pb.finish_with_message(format!("Done: {f}"));
                        }
                    }
                } else {
                    let new_filename = substitute_filename(filename, &rhai_engine, liquid_object)?;
                    let relative_path = new_filename.strip_prefix(project_dir)?;
                    let f = relative_path.display();
                    pb.inc(50);
                    if filename != new_filename {
                        fs::remove_dir_all(filename)?;
                    }
                    pb.inc(50);
                    pb.finish_with_message(format!("Done: {f}"));
                }
            }
            ShouldInclude::Exclude => {
                pb.finish_with_message(format!("Skipped: {filename_display}"));
            }
            ShouldInclude::Ignore => {
                pb.finish_with_message(format!("Ignored: {filename_display}"));
            }
        }
    }

    if !files_with_errors.is_empty() {
        print_files_with_errors_warning(files_with_errors);
    }

    Ok(())
}

fn template_process_file(
    context: &LiquidObjectResource,
    parser: &Parser,
    file: &Path,
) -> liquid_core::Result<String> {
    let content =
        fs::read_to_string(file).map_err(|e| liquid_core::Error::with_msg(e.to_string()))?;
    render_string_gracefully(context, parser, content.as_str())
}

pub fn render_string_gracefully(
    context: &LiquidObjectResource,
    parser: &Parser,
    content: &str,
) -> liquid_core::Result<String> {
    let template = parser.parse(content)?;

    // Liquid engine needs access to the context.
    // At the same time, our own `rhai` liquid filter may also need it, but doesn't have access
    // to the one provided to the liquid engine, thus it has it's own cloned `Arc` for it. These
    // WILL collide and cause the `Mutex` to hang in case the user tries to modify any variable
    // inside a rhai filter script - so we currently clone it, and let any rhai filter manipulate
    // the original. Note that hooks do not run at the same time as liquid, thus they do not
    // suffer these limitations.
    let render_object_view = {
        let ref_cell = context
            .lock()
            .map_err(|_| liquid_core::Error::with_msg(PoisonError.to_string()))?;
        let object_view = ref_cell.borrow();
        object_view.to_owned()
    };
    let render_result = template.render(&render_object_view);

    match render_result {
        ctx @ Ok(_) => ctx,
        Err(e) => {
            // handle it gracefully
            let msg = e.to_string();
            if msg.contains("requested variable") {
                // so, we miss a variable that is present in the file to render
                let requested_var =
                    regex::Regex::new(r"(?P<p>.*requested\svariable=)(?P<v>.*)").unwrap();
                let captures = requested_var.captures(msg.as_str()).unwrap();
                if let Some(Some(req_var)) = captures.iter().last() {
                    let missing_variable = KString::from(req_var.as_str().to_string());
                    // The missing variable might have been supplied by a rhai filter,
                    // if not, substitute an empty string before retrying
                    let _ = context
                        .lock()
                        .map_err(|_| liquid_core::Error::with_msg(PoisonError.to_string()))?
                        .borrow_mut()
                        .entry(missing_variable)
                        .or_insert_with(|| Value::scalar("".to_string()));
                    return render_string_gracefully(context, parser, content);
                }
            }
            // todo: find nice way to have this happening outside of this fn
            // error!(
            //     "{} `{}`",
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
        "\n{}",
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

    warn!("{msg}\n{hint}\n\n{read_more}");
}
