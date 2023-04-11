#![doc = include_str!("../README.md")]
#![warn(
    //clippy::cargo_common_metadata,
    clippy::branches_sharing_code,
    clippy::cast_lossless,
    clippy::cognitive_complexity,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::inefficient_to_string,
    clippy::match_bool,
    clippy::missing_const_for_fn,
    clippy::missing_panics_doc,
    clippy::option_if_let_else,
    clippy::redundant_closure,
    clippy::redundant_else,
    clippy::redundant_pub_crate,
    clippy::ref_binding_to_reference,
    clippy::ref_option_ref,
    clippy::same_functions_in_if_condition,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::use_self,
)]

mod app_config;
mod args;
mod config;
mod emoji;
mod favorites;
mod filenames;
mod git;
mod hooks;
mod ignore_me;
mod include_exclude;
mod interactive;
mod progressbar;
mod project_variables;
mod template;
mod template_filters;
mod template_variables;
mod user_parsed_input;

pub use crate::app_config::{app_config_path, AppConfig};
pub use crate::favorites::list_favorites;
use crate::template::create_liquid_engine;
pub use args::*;

use anyhow::{anyhow, bail, Context, Result};
use config::{locate_template_configs, Config, CONFIG_FILE_NAME};
use console::style;
use env_logger::fmt::Formatter;
use hooks::execute_hooks;
use ignore_me::remove_dir_files;
use interactive::prompt_and_check_variable;
use log::Record;
use log::{info, warn};
use project_variables::{StringEntry, TemplateSlots, VarInfo};
use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tempfile::TempDir;
use user_parsed_input::{TemplateLocation, UserParsedInput};

use crate::template_variables::{
    load_env_and_args_template_values, CrateName, ProjectDir, ProjectNameInput,
};
use crate::{project_variables::ConversionError, template_variables::ProjectName};

use self::config::TemplateConfig;
use self::git::try_get_branch_from_path;
use self::hooks::evaluate_script;
use self::template::{create_liquid_object, set_project_name_variables, LiquidObjectResource};

/// Logging formatter function
pub fn log_formatter(
    buf: &mut Formatter,
    record: &Record,
) -> std::result::Result<(), std::io::Error> {
    let prefix = match record.level() {
        log::Level::Error => format!("{} ", emoji::ERROR),
        log::Level::Warn => format!("{} ", emoji::WARN),
        _ => "".to_string(),
    };

    writeln!(buf, "{}{}", prefix, record.args())
}

/// # Panics
pub fn generate(args: GenerateArgs) -> Result<PathBuf> {
    let _working_dir_scope = ScopedWorkingDirectory::default();

    let app_config = AppConfig::try_from(app_config_path(&args.config)?.as_path())?;

    // mash AppConfig and CLI arguments together into UserParsedInput
    let mut user_parsed_input = UserParsedInput::try_from_args_and_config(app_config, &args);
    // let ENV vars provide values we don't have yet
    user_parsed_input
        .template_values_mut()
        .extend(load_env_and_args_template_values(&args)?);

    let (template_base_dir, template_dir, branch) = prepare_local_template(&user_parsed_input)?;

    // read configuration in the template
    let mut config = Config::from_path(
        &locate_template_file(CONFIG_FILE_NAME, &template_base_dir, &template_dir).ok(),
    )?;

    // the `--init` parameter may also be set by the template itself
    if config
        .template
        .as_ref()
        .and_then(|c| c.init)
        .unwrap_or(false)
        && !user_parsed_input.init
    {
        warn!(
            "{}",
            style("Template specifies --init, while not specified on the command line. Output location is affected!").bold().red(),
        );

        user_parsed_input.init = true;
    };

    check_cargo_generate_version(&config)?;

    let project_dir = expand_template(&template_dir, &mut config, &user_parsed_input, &args)?;

    if user_parsed_input.test() {
        test_expanded_template(&template_dir, args.other_args)
    } else {
        copy_expanded_template(
            template_dir,
            project_dir,
            user_parsed_input,
            config,
            branch.as_deref(),
        )
    }
}

fn copy_expanded_template(
    template_dir: PathBuf,
    project_dir: PathBuf,
    user_parsed_input: UserParsedInput,
    config: Config,
    branch: Option<&str>,
) -> Result<PathBuf> {
    info!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Moving generated files into:").bold(),
        style(project_dir.display()).bold().yellow(),
        style("...").bold()
    );
    copy_dir_all(template_dir, &project_dir, user_parsed_input.overwrite())?;
    let vcs = config
        .template
        .and_then(|t| t.vcs)
        .unwrap_or_else(|| user_parsed_input.vcs());
    if !vcs.is_none() && (!user_parsed_input.init || user_parsed_input.force_git_init()) {
        info!("{}", style("Initializing a fresh Git repository").bold());
        vcs.initialize(&project_dir, branch, user_parsed_input.force_git_init())?;
    }
    info!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(&project_dir.display()).underlined()
    );
    Ok(project_dir)
}

fn test_expanded_template(template_dir: &PathBuf, args: Option<Vec<String>>) -> Result<PathBuf> {
    info!(
        "{} {}{}{}",
        emoji::WRENCH,
        style("Running \"").bold(),
        style("cargo test"),
        style("\" ...").bold(),
    );
    std::env::set_current_dir(template_dir)?;
    let (cmd, cmd_args) = std::env::var("CARGO_GENERATE_TEST_CMD").map_or_else(
        |_| (String::from("cargo"), vec![String::from("test")]),
        |env_test_cmd| {
            let mut split_cmd_args = env_test_cmd.split_whitespace().map(str::to_string);
            (
                split_cmd_args.next().unwrap(),
                split_cmd_args.collect::<Vec<String>>(),
            )
        },
    );
    std::process::Command::new(cmd)
        .args(cmd_args)
        .args(args.unwrap_or_default().into_iter())
        .spawn()?
        .wait()?
        .success()
        .then(PathBuf::new)
        .ok_or_else(|| anyhow!("{} Testing failed", emoji::ERROR))
}

fn prepare_local_template(
    source_template: &UserParsedInput,
) -> Result<(TempDir, PathBuf, Option<String>), anyhow::Error> {
    let (temp_dir, branch) = get_source_template_into_temp(source_template.location())?;
    let template_folder = resolve_template_dir(&temp_dir, source_template.subfolder())?;

    Ok((temp_dir, template_folder, branch))
}

fn get_source_template_into_temp(
    template_location: &TemplateLocation,
) -> Result<(TempDir, Option<String>)> {
    match template_location {
        TemplateLocation::Git(git) => {
            let result = git::clone_git_template_into_temp(
                git.url(),
                git.branch(),
                git.tag(),
                git.identity(),
            )
            .map(|(dir, branch)| (dir, Some(branch)));
            if let Ok((ref temp_dir, _)) = result {
                git::remove_history(temp_dir.path())?;
                strip_liquid_suffixes(temp_dir.path())?;
            };
            result
        }
        TemplateLocation::Path(path) => {
            let temp_dir = tempfile::tempdir()?;
            copy_dir_all(path, temp_dir.path(), false)?;
            git::remove_history(temp_dir.path())?;
            Ok((temp_dir, try_get_branch_from_path(path)))
        }
    }
}

/// remove .liquid suffixes from git templates for parity with path templates
fn strip_liquid_suffixes(dir: impl AsRef<Path>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;

        if entry_type.is_dir() {
            strip_liquid_suffixes(entry.path())?;
        } else if entry_type.is_file() {
            let path = entry.path().to_string_lossy().to_string();
            if let Some(new_path) = path.clone().strip_suffix(".liquid") {
                fs::rename(path, new_path)?;
            }
        }
    }
    Ok(())
}

/// resolve the template location for the actual template to expand
fn resolve_template_dir(template_base_dir: &TempDir, subfolder: Option<&str>) -> Result<PathBuf> {
    let template_dir = resolve_template_dir_subfolder(template_base_dir.path(), subfolder)?;
    auto_locate_template_dir(template_dir, &mut |slots| {
        prompt_and_check_variable(slots, None)
    })
}

/// join the base-dir and the sufolder, ensuring that we stay within the template directory
fn resolve_template_dir_subfolder(
    template_base_dir: &Path,
    subfolder: Option<impl AsRef<str>>,
) -> Result<PathBuf> {
    if let Some(subfolder) = subfolder {
        let template_base_dir = fs::canonicalize(template_base_dir)?;
        let template_dir = fs::canonicalize(template_base_dir.join(subfolder.as_ref()))
            .with_context(|| {
                format!(
                    "not able to find subfolder '{}' in source template",
                    subfolder.as_ref()
                )
            })?;

        // make sure subfolder is not `../../subfolder`
        if !template_dir.starts_with(&template_base_dir) {
            return Err(anyhow!(
                "{} {} {}",
                emoji::ERROR,
                style("Subfolder Error:").bold().red(),
                style("Invalid subfolder. Must be part of the template folder structure.")
                    .bold()
                    .red(),
            ));
        }

        if !template_dir.is_dir() {
            return Err(anyhow!(
                "{} {} {}",
                emoji::ERROR,
                style("Subfolder Error:").bold().red(),
                style("The specified subfolder must be a valid folder.")
                    .bold()
                    .red(),
            ));
        }

        Ok(template_dir)
    } else {
        Ok(template_base_dir.to_owned())
    }
}

/// look through the template folder structure and attempt to find a suitable template.
fn auto_locate_template_dir(
    template_base_dir: PathBuf,
    prompt: &mut impl FnMut(&TemplateSlots) -> Result<String>,
) -> Result<PathBuf> {
    let config_paths = locate_template_configs(&template_base_dir)?;
    match config_paths.len() {
        0 => {
            // No configurations found, so this *must* be a template
            Ok(template_base_dir)
        }
        1 => {
            // A single configuration found, but it may contain multiple configured sub-templates
            resolve_configured_sub_templates(&template_base_dir.join(&config_paths[0]), prompt)
        }
        _ => {
            // Multiple configurations found, each in different "roots"
            // let user select between them
            let prompt_args = TemplateSlots {
                prompt: "Which template should be expanded?".into(),
                var_name: "Template".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some(config_paths[0].display().to_string()),
                        choices: Some(
                            config_paths
                                .into_iter()
                                .map(|p| p.display().to_string())
                                .collect(),
                        ),
                        regex: None,
                    }),
                },
            };
            let path = prompt(&prompt_args)?;

            // recursively retry to resolve the template,
            // until we hit a single or no config, idetifying the final template folder
            auto_locate_template_dir(template_base_dir.join(path), prompt)
        }
    }
}

fn resolve_configured_sub_templates(
    config_path: &Path,
    prompt: &mut impl FnMut(&TemplateSlots) -> Result<String>,
) -> Result<PathBuf> {
    Config::from_path(&Some(config_path.join(CONFIG_FILE_NAME)))
        .ok()
        .and_then(|config| config.template)
        .and_then(|config| config.sub_templates)
        .map_or_else(
            || Ok(PathBuf::from(config_path)),
            |sub_templates| {
                // we have a config that defines sub-templates, let the user select
                let prompt_args = TemplateSlots {
                    prompt: "Which sub-template should be expanded?".into(),
                    var_name: "Template".into(),
                    var_info: VarInfo::String {
                        entry: Box::new(StringEntry {
                            default: Some(sub_templates[0].clone()),
                            choices: Some(sub_templates.clone()),
                            regex: None,
                        }),
                    },
                };
                let path = prompt(&prompt_args)?;

                // recursively retry to resolve the template,
                // until we hit a single or no config, idetifying the final template folder
                auto_locate_template_dir(
                    resolve_template_dir_subfolder(config_path, Some(path))?,
                    prompt,
                )
            },
        )
}

pub(crate) fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    overwrite: bool,
) -> Result<()> {
    fn check_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>, overwrite: bool) -> Result<()> {
        if !dst.as_ref().exists() {
            return Ok(());
        }

        for src_entry in fs::read_dir(src)? {
            let src_entry = src_entry?;
            let filename = src_entry.file_name().to_string_lossy().to_string();
            let entry_type = src_entry.file_type()?;

            if entry_type.is_dir() {
                if filename == ".git" {
                    continue;
                }
                let dst_path = dst.as_ref().join(filename);
                check_dir_all(src_entry.path(), dst_path, overwrite)?;
            } else if entry_type.is_file() {
                let filename = filename.strip_suffix(".liquid").unwrap_or(&filename);
                let dst_path = dst.as_ref().join(filename);
                match (dst_path.exists(), overwrite) {
                    (true, false) => {
                        bail!(
                            "{} {} {}",
                            crate::emoji::WARN,
                            style("File already exists:").bold().red(),
                            style(dst_path.display()).bold().red(),
                        )
                    }
                    (true, true) => {
                        warn!(
                            "{} {}",
                            style("Overwriting file:").bold().red(),
                            style(dst_path.display()).bold().red(),
                        );
                    }
                    _ => {}
                };
            } else {
                bail!(
                    "{} {}",
                    crate::emoji::WARN,
                    style("Symbolic links not supported").bold().red(),
                )
            }
        }
        Ok(())
    }
    fn copy_all(src: impl AsRef<Path>, dst: impl AsRef<Path>, overwrite: bool) -> Result<()> {
        fs::create_dir_all(&dst)?;
        for src_entry in fs::read_dir(src)? {
            let src_entry = src_entry?;
            let filename = src_entry.file_name().to_string_lossy().to_string();
            let entry_type = src_entry.file_type()?;
            if entry_type.is_dir() {
                let dst_path = dst.as_ref().join(filename);
                if ".git" == src_entry.file_name() {
                    continue;
                }
                copy_dir_all(src_entry.path(), dst_path, overwrite)?;
            } else if entry_type.is_file() {
                let filename = filename.strip_suffix(".liquid").unwrap_or(&filename);
                let dst_path = dst.as_ref().join(filename);
                if dst_path.exists() && overwrite {
                    fs::remove_file(&dst_path)?;
                }
                fs::copy(src_entry.path(), dst_path)?;
            }
        }
        Ok(())
    }

    check_dir_all(&src, &dst, overwrite)?;
    copy_all(src, dst, overwrite)
}

fn locate_template_file(
    name: &str,
    template_base_folder: impl AsRef<Path>,
    template_folder: impl AsRef<Path>,
) -> Result<PathBuf> {
    let template_base_folder = template_base_folder.as_ref();
    let mut search_folder = template_folder.as_ref().to_path_buf();
    loop {
        let file_path = search_folder.join(name.borrow());
        if file_path.exists() {
            return Ok(file_path);
        }
        if search_folder == template_base_folder {
            bail!("File not found within template");
        }
        search_folder = search_folder
            .parent()
            .ok_or_else(|| anyhow!("Reached root folder"))?
            .to_path_buf();
    }
}

fn expand_template(
    template_dir: &Path,
    config: &mut Config,
    user_parsed_input: &UserParsedInput,
    args: &GenerateArgs,
) -> Result<PathBuf> {
    let liquid_object = create_liquid_object(user_parsed_input)?;

    // run init hooks - these won't have access to `crate_name`/`within_cargo_project`
    // variables, as these are not set yet. Furthermore, if `project-name` is set, it is the raw
    // user input!
    // The init hooks are free to set `project-name` (but it will be validated before further
    // use).
    execute_hooks(
        template_dir,
        &liquid_object,
        &config.get_init_hooks(),
        user_parsed_input.allow_commands(),
        user_parsed_input.silent(),
    )?;

    let project_name_input = ProjectNameInput::try_from((&liquid_object, user_parsed_input))?;
    let destination = ProjectDir::try_from((&project_name_input, user_parsed_input))?;
    let project_name = ProjectName::from((&project_name_input, user_parsed_input));
    let crate_name = CrateName::from(&project_name_input);
    set_project_name_variables(&liquid_object, &destination, &project_name, &crate_name)?;

    info!(
        "{} {} {}",
        emoji::WRENCH,
        style(format!("Destination: {destination}")).bold(),
        style("...").bold()
    );
    info!(
        "{} {} {}",
        emoji::WRENCH,
        style(format!("project-name: {project_name}")).bold(),
        style("...").bold()
    );
    info!(
        "{} {} {}",
        emoji::WRENCH,
        style("Generating template").bold(),
        style("...").bold()
    );

    // evaluate config for placeholders and and any that are undefined
    fill_placeholders_and_merge_conditionals(
        config,
        &liquid_object,
        user_parsed_input.template_values(),
        args,
    )?;
    add_missing_provided_values(&liquid_object, user_parsed_input.template_values())?;

    // run pre-hooks
    execute_hooks(
        template_dir,
        &liquid_object,
        &config.get_pre_hooks(),
        user_parsed_input.allow_commands(),
        user_parsed_input.silent(),
    )?;

    // walk/evaluate the template
    let all_hook_files = config.get_hook_files();
    let mut template_config = config.template.take().unwrap_or_default();

    ignore_me::remove_unneeded_files(template_dir, &template_config.ignore, args.verbose)?;
    let mut pbar = progressbar::new();

    let rhai_filter_files = Arc::new(Mutex::new(vec![]));
    let rhai_engine = create_liquid_engine(
        template_dir.to_owned(),
        liquid_object.clone(),
        user_parsed_input.allow_commands(),
        user_parsed_input.silent(),
        rhai_filter_files.clone(),
    );
    template::walk_dir(
        &mut template_config,
        template_dir,
        &all_hook_files,
        &liquid_object,
        rhai_engine,
        &rhai_filter_files,
        &mut pbar,
        args.verbose,
    )?;

    // run post-hooks
    execute_hooks(
        template_dir,
        &liquid_object,
        &config.get_post_hooks(),
        user_parsed_input.allow_commands(),
        user_parsed_input.silent(),
    )?;

    // remove all hook and filter files as they are never part of the template output
    let rhai_filter_files = rhai_filter_files
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    remove_dir_files(
        all_hook_files
            .into_iter()
            .map(PathBuf::from)
            .chain(rhai_filter_files),
        false,
    );

    config.template.replace(template_config);
    Ok(destination.as_ref().to_owned())
}

/// Try to add all provided `template_values` to the `liquid_object`.
///
/// ## Note:
/// Values for which a placeholder exists, should already be filled by `fill_project_variables`
pub(crate) fn add_missing_provided_values(
    liquid_object: &LiquidObjectResource,
    template_values: &HashMap<String, toml::Value>,
) -> Result<(), anyhow::Error> {
    template_values.iter().try_for_each(|(k, v)| {
        if RefCell::borrow(&liquid_object.lock().unwrap()).contains_key(k.as_str()) {
            return Ok(());
        }
        // we have a value without a slot in the liquid object.
        // try to create the slot from the provided value
        let value = match v {
            toml::Value::String(content) => liquid_core::Value::Scalar(content.clone().into()),
            toml::Value::Boolean(content) => liquid_core::Value::Scalar((*content).into()),
            _ => anyhow::bail!(format!(
                "{} {}",
                emoji::ERROR,
                style("Unsupported value type. Only Strings and Booleans are supported.")
                    .bold()
                    .red(),
            )),
        };
        liquid_object
            .lock()
            .unwrap()
            .borrow_mut()
            .insert(k.clone().into(), value);
        Ok(())
    })?;
    Ok(())
}

// Evaluate the configuration, adding defined placeholder variables to the liquid object.
fn fill_placeholders_and_merge_conditionals(
    config: &mut Config,
    liquid_object: &LiquidObjectResource,
    template_values: &HashMap<String, toml::Value>,
    args: &GenerateArgs,
) -> Result<()> {
    let mut conditionals = config.conditional.take().unwrap_or_default();

    loop {
        // keep evaluating for placeholder variables as long new ones are added.
        project_variables::fill_project_variables(liquid_object, config, |slot| {
            let provided_value = template_values.get(&slot.var_name).and_then(|v| match v {
                toml::Value::String(s) => Some(s.clone()),
                toml::Value::Integer(s) => Some(s.to_string()),
                toml::Value::Float(s) => Some(s.to_string()),
                toml::Value::Boolean(s) => Some(s.to_string()),
                toml::Value::Datetime(s) => Some(s.to_string()),
                toml::Value::Array(_) | toml::Value::Table(_) => None,
            });
            if provided_value.is_none() && args.silent {
                anyhow::bail!(ConversionError::MissingPlaceholderVariable {
                    var_name: slot.var_name.clone()
                })
            }
            interactive::variable(slot, provided_value.as_ref())
        })?;

        let placeholders_changed = conditionals
            .iter_mut()
            // filter each conditional config block by trueness of the expression, given the known variables
            .filter_map(|(key, cfg)| {
                evaluate_script::<bool>(liquid_object, key)
                    .ok()
                    .filter(|&r| r)
                    .map(|_| cfg)
            })
            .map(|conditional_template_cfg| {
                // append the conditional blocks configuration, returning true if any placeholders were added
                let template_cfg = config.template.get_or_insert_with(TemplateConfig::default);
                if let Some(mut extras) = conditional_template_cfg.include.take() {
                    template_cfg
                        .include
                        .get_or_insert_with(Vec::default)
                        .append(&mut extras);
                }
                if let Some(mut extras) = conditional_template_cfg.exclude.take() {
                    template_cfg
                        .exclude
                        .get_or_insert_with(Vec::default)
                        .append(&mut extras);
                }
                if let Some(mut extras) = conditional_template_cfg.ignore.take() {
                    template_cfg
                        .ignore
                        .get_or_insert_with(Vec::default)
                        .append(&mut extras);
                }
                if let Some(extra_placeholders) = conditional_template_cfg.placeholders.take() {
                    match config.placeholders.as_mut() {
                        Some(placeholders) => {
                            for (k, v) in extra_placeholders.0 {
                                placeholders.0.insert(k, v);
                            }
                        }
                        None => {
                            config.placeholders = Some(extra_placeholders);
                        }
                    };
                    return true;
                }
                false
            })
            .fold(false, |acc, placeholders_changed| {
                acc | placeholders_changed
            });

        if !placeholders_changed {
            break;
        }
    }

    Ok(())
}

fn check_cargo_generate_version(template_config: &Config) -> Result<(), anyhow::Error> {
    if let Config {
        template:
            Some(config::TemplateConfig {
                cargo_generate_version: Some(requirement),
                ..
            }),
        ..
    } = template_config
    {
        let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
        if !requirement.matches(&version) {
            bail!(
                "{} {} {} {} {}",
                emoji::ERROR,
                style("Required cargo-generate version not met. Required:")
                    .bold()
                    .red(),
                style(requirement).yellow(),
                style(" was:").bold().red(),
                style(version).yellow(),
            );
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ScopedWorkingDirectory(PathBuf);

impl Default for ScopedWorkingDirectory {
    fn default() -> Self {
        Self(env::current_dir().unwrap())
    }
}

impl Drop for ScopedWorkingDirectory {
    fn drop(&mut self) {
        env::set_current_dir(&self.0).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::{auto_locate_template_dir, project_variables::VarInfo};
    use anyhow::anyhow;
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
    };
    use tempfile::{tempdir, TempDir};

    #[test]
    fn auto_locate_template_returns_base_when_no_cargo_generate_is_found() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;

        let actual =
            auto_locate_template_dir(tmp.path().to_path_buf(), &mut |_slots| Err(anyhow!("test")))?
                .canonicalize()?;
        let expected = tmp.path().canonicalize()?;

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn auto_locate_template_returns_path_when_single_cargo_generate_is_found() -> anyhow::Result<()>
    {
        let tmp = tempdir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_2/cargo-generate.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;

        let actual =
            auto_locate_template_dir(tmp.path().to_path_buf(), &mut |_slots| Err(anyhow!("test")))?
                .canonicalize()?;
        let expected = tmp.path().join("dir2/dir2_2").canonicalize()?;

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn auto_locate_template_can_resolve_configured_subtemplates() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        create_file(
            &tmp,
            "cargo-generate.toml",
            indoc::indoc! {r#"
                [template]
                sub_templates = ["sub1", "sub2"]
            "#},
        )?;
        create_file(&tmp, "sub1/Cargo.toml", "")?;
        create_file(&tmp, "sub2/Cargo.toml", "")?;

        let actual = auto_locate_template_dir(tmp.path().to_path_buf(), &mut |slots| match &slots
            .var_info
        {
            VarInfo::Bool { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let Some(choices) = entry.choices.clone() {
                    let expected = vec!["sub1".to_string(), "sub2".to_string()];
                    assert_eq!(expected, choices);
                    Ok("sub2".to_string())
                } else {
                    anyhow::bail!("Missing choices")
                }
            }
        })?
        .canonicalize()?;
        let expected = tmp.path().join("sub2").canonicalize()?;

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn auto_locate_template_recurses_to_resolve_subtemplates() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        create_file(
            &tmp,
            "cargo-generate.toml",
            indoc::indoc! {r#"
                [template]
                sub_templates = ["sub1", "sub2"]
            "#},
        )?;
        create_file(&tmp, "sub1/Cargo.toml", "")?;
        create_file(&tmp, "sub1/sub11/cargo-generate.toml", "")?;
        create_file(
            &tmp,
            "sub1/sub12/cargo-generate.toml",
            indoc::indoc! {r#"
                [template]
                sub_templates = ["sub122", "sub121"]
            "#},
        )?;
        create_file(&tmp, "sub2/Cargo.toml", "")?;
        create_file(&tmp, "sub1/sub11/Cargo.toml", "")?;
        create_file(&tmp, "sub1/sub12/sub121/Cargo.toml", "")?;
        create_file(&tmp, "sub1/sub12/sub122/Cargo.toml", "")?;

        let mut prompt_num = 0;
        let actual = auto_locate_template_dir(tmp.path().to_path_buf(), &mut |slots| match &slots
            .var_info
        {
            VarInfo::Bool { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let Some(choices) = entry.choices.clone() {
                    let (expected, answer) = match prompt_num {
                        0 => (vec!["sub1", "sub2"], "sub1"),
                        1 => (vec!["sub11", "sub12"], "sub12"),
                        2 => (vec!["sub122", "sub121"], "sub121"),
                        _ => panic!("Unexpected number of prompts"),
                    };
                    prompt_num += 1;
                    expected
                        .into_iter()
                        .zip(choices.iter())
                        .for_each(|(a, b)| assert_eq!(a, b));
                    Ok(answer.to_string())
                } else {
                    anyhow::bail!("Missing choices")
                }
            }
        })?
        .canonicalize()?;

        let expected = tmp
            .path()
            .join("sub1")
            .join("sub12")
            .join("sub121")
            .canonicalize()?;

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn auto_locate_template_prompts_when_multiple_cargo_generate_is_found() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_2/cargo-generate.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;
        create_file(&tmp, "dir4/cargo-generate.toml", "")?;

        let actual = auto_locate_template_dir(tmp.path().to_path_buf(), &mut |slots| match &slots
            .var_info
        {
            VarInfo::Bool { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let Some(choices) = entry.choices.clone() {
                    let expected = vec![
                        Path::new("dir2").join("dir2_2").to_string(),
                        "dir4".to_string(),
                    ];
                    assert_eq!(expected, choices);
                    Ok("dir4".to_string())
                } else {
                    anyhow::bail!("Missing choices")
                }
            }
        })?
        .canonicalize()?;
        let expected = tmp.path().join("dir4").canonicalize()?;

        assert_eq!(expected, actual);

        Ok(())
    }

    pub trait PathString {
        fn to_string(&self) -> String;
    }

    impl PathString for PathBuf {
        fn to_string(&self) -> String {
            self.as_path().to_string()
        }
    }

    impl PathString for Path {
        fn to_string(&self) -> String {
            self.display().to_string()
        }
    }

    pub fn create_file(
        base_path: &TempDir,
        path: impl AsRef<Path>,
        contents: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let path = base_path.path().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::File::create(&path)?.write_all(contents.as_ref().as_ref())?;
        Ok(())
    }
}
