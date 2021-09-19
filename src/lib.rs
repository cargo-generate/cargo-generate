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
mod log;
mod progressbar;
mod project_variables;
mod template;
mod template_filters;
mod template_variables;

pub use args::*;

use anyhow::{anyhow, bail, Context, Result};
use config::{locate_template_configs, Config, CONFIG_FILE_NAME};
use console::style;
use favorites::{list_favorites, resolve_favorite_args_and_default_values};
use hooks::{execute_post_hooks, execute_pre_hooks};
use ignore_me::remove_dir_files;
use interactive::prompt_for_variable;
use liquid::ValueView;
use project_variables::{StringEntry, TemplateSlots, VarInfo};
use std::{
    borrow::Borrow,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use crate::{
    app_config::{app_config_path, AppConfig},
    project_variables::ConversionError,
    template_variables::{CrateType, ProjectName},
};
use crate::{git::GitConfig, template_variables::resolve_template_values};

pub fn generate(mut args: Args) -> Result<()> {
    let app_config = AppConfig::from_path(&app_config_path(&args.config)?)?;

    if args.list_favorites {
        return list_favorites(&app_config, &args);
    }

    let default_values = resolve_favorite_args_and_default_values(&app_config, &mut args)?;

    let project_name = resolve_project_name(&args)?;
    let project_dir = resolve_project_dir(&project_name, &args)?;

    let (template_base_dir, template_folder, branch) = prepare_local_template(&args)?;

    let template_config = Config::from_path(
        &locate_template_file(CONFIG_FILE_NAME, &template_base_dir, &args.subfolder).ok(),
    )?
    .unwrap_or_default();

    check_cargo_generate_version(&template_config)?;
    let template_values = resolve_template_values(default_values, &args)?;

    println!(
        "{} {} {}",
        emoji::WRENCH,
        style("Generating template").bold(),
        style("...").bold()
    );

    expand_template(
        &project_name,
        &template_folder,
        &template_values,
        template_config,
        &args,
    )?;

    println!(
        "{} {} `{}`{}",
        emoji::WRENCH,
        style("Moving generated files into:").bold(),
        style(project_dir.display()).bold().yellow(),
        style("...").bold()
    );
    copy_dir_all(&template_folder, &project_dir)?;

    if !args.init {
        args.vcs.initialize(&project_dir, branch)?;
    }

    println!(
        "{} {} {} {}",
        emoji::SPARKLE,
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(&project_dir.display()).underlined()
    );
    Ok(())
}

fn prepare_local_template(args: &Args) -> Result<(TempDir, PathBuf, String), anyhow::Error> {
    let (template_base_dir, template_folder, branch) = match (&args.git, &args.path) {
        (Some(_), None) => {
            let (template_base_dir, branch) = clone_git_template_into_temp(args)?;
            let template_folder = resolve_template_dir(&template_base_dir, args)?;
            (template_base_dir, template_folder, branch)
        }
        (None, Some(_)) => {
            let template_base_dir = copy_path_template_into_temp(args)?;
            let branch = args.branch.clone().unwrap_or_else(|| String::from("main"));
            let template_folder =
                auto_locate_template_dir(template_base_dir.path(), prompt_for_variable)?;
            (template_base_dir, template_folder, branch)
        }
        _ => bail!(
            "{} {} {} {} {}",
            emoji::ERROR,
            style("Please specify either").bold(),
            style("--git <repo>").bold().yellow(),
            style("or").bold(),
            style("--path <path>").bold().yellow(),
        ),
    };

    Ok((template_base_dir, template_folder, branch))
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

fn resolve_project_name(args: &Args) -> Result<ProjectName> {
    match args.name {
        Some(ref n) => Ok(ProjectName::new(n)),
        None if !args.silent => Ok(ProjectName::new(interactive::name()?)),
        None => Err(anyhow!(
            "{} {} {}",
            emoji::ERROR,
            style("Project Name Error:").bold().red(),
            style("Option `--silent` provided, but project name was not set. Please use `--name`.")
                .bold()
                .red(),
        )),
    }
}

fn resolve_template_dir(template_base_dir: &TempDir, args: &Args) -> Result<PathBuf> {
    match &args.subfolder {
        Some(subfolder) => {
            let template_base_dir = fs::canonicalize(template_base_dir.path())?;
            let template_dir = fs::canonicalize(template_base_dir.join(subfolder))?;

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

            Ok(auto_locate_template_dir(
                &template_dir,
                prompt_for_variable,
            )?)
        }
        None => auto_locate_template_dir(template_base_dir.path(), prompt_for_variable),
    }
}

fn auto_locate_template_dir(
    template_base_dir: &Path,
    prompt: impl Fn(&TemplateSlots) -> Result<String>,
) -> Result<PathBuf> {
    let config_paths = locate_template_configs(template_base_dir)?;
    match config_paths.len() {
        0 => Ok(template_base_dir.to_owned()),
        1 => Ok(template_base_dir.join(&config_paths[0])),
        _ => {
            let prompt_args = TemplateSlots {
                prompt: "Which template should be expanded?".into(),
                var_name: "Template".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some(config_paths[0].clone()),
                        choices: Some(config_paths),
                        regex: None,
                    }),
                },
            };
            let path = prompt(&prompt_args)?;
            Ok(template_base_dir.join(&path))
        }
    }
}

fn copy_path_template_into_temp(args: &Args) -> Result<TempDir> {
    let path_clone_dir = tempfile::tempdir()?;
    copy_dir_all(
        args.path
            .as_ref()
            .with_context(|| "Missing option git, path or a favorite")?,
        path_clone_dir.path(),
    )?;
    Ok(path_clone_dir)
}

fn clone_git_template_into_temp(args: &Args) -> Result<(TempDir, String)> {
    let git_clone_dir = tempfile::tempdir()?;

    let remote = args
        .git
        .clone()
        .with_context(|| "Missing option git, path or a favorite")?;

    let git_config = GitConfig::new_abbr(
        remote.into(),
        args.branch.to_owned(),
        args.ssh_identity.clone(),
    )?;

    let branch = git::create(git_clone_dir.path(), git_config).map_err(|e| {
        anyhow!(
            "{} {} {}",
            emoji::ERROR,
            style("Git Error:").bold().red(),
            style(e).bold().red(),
        )
    })?;

    Ok((git_clone_dir, branch))
}

pub(crate) fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fn check_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        if !dst.as_ref().exists() {
            return Ok(());
        }

        for src_entry in fs::read_dir(src)? {
            let src_entry = src_entry?;
            let dst_path = dst.as_ref().join(src_entry.file_name());
            let entry_type = src_entry.file_type()?;

            if entry_type.is_dir() {
                check_dir_all(src_entry.path(), dst_path)?;
            } else if entry_type.is_file() {
                if dst_path.exists() {
                    bail!(
                        "{} {} {}",
                        crate::emoji::WARN,
                        style("File already exists:").bold().red(),
                        style(dst_path.display()).bold().red(),
                    )
                }
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
    fn copy_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        fs::create_dir_all(&dst)?;
        for src_entry in fs::read_dir(src)? {
            let src_entry = src_entry?;
            let dst_path = dst.as_ref().join(src_entry.file_name());
            let entry_type = src_entry.file_type()?;
            if entry_type.is_dir() {
                copy_dir_all(src_entry.path(), dst_path)?;
            } else if entry_type.is_file() {
                fs::copy(src_entry.path(), dst_path)?;
            }
        }
        Ok(())
    }

    check_dir_all(&src, &dst)?;
    copy_all(src, dst)
}

fn locate_template_file<T>(
    name: &str,
    template_folder: T,
    subfolder: &Option<String>,
) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let template_folder = template_folder.as_ref().to_path_buf();
    let mut search_folder = subfolder
        .as_ref()
        .map_or_else(|| template_folder.to_owned(), |s| template_folder.join(s));
    loop {
        let file_path = search_folder.join(name.borrow());
        if file_path.exists() {
            return Ok(file_path);
        }
        if search_folder == template_folder {
            bail!("File not found within template");
        }
        search_folder = search_folder
            .parent()
            .ok_or_else(|| anyhow!("Reached root folder"))?
            .to_path_buf();
    }
}

fn resolve_project_dir(name: &ProjectName, args: &Args) -> Result<PathBuf> {
    if args.init {
        let cwd = env::current_dir()?;
        return Ok(cwd);
    }

    let dir_name = if args.force {
        name.raw()
    } else {
        rename_warning(name);
        name.kebab_case()
    };
    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&dir_name);

    if project_dir.exists() {
        Err(anyhow!(
            "{} {}",
            emoji::ERROR,
            style("Target directory already exists, aborting!")
                .bold()
                .red()
        ))
    } else {
        Ok(project_dir)
    }
}

fn expand_template(
    name: &ProjectName,
    dir: &Path,
    template_values: &HashMap<String, toml::Value>,
    mut template_config: Config,
    args: &Args,
) -> Result<()> {
    let crate_type: CrateType = args.into();
    let liquid_object = template::create_liquid_object(name, &crate_type, args.force)?;
    let liquid_object =
        project_variables::fill_project_variables(liquid_object, &template_config, |slot| {
            let provided_value = template_values.get(&slot.var_name).and_then(|v| v.as_str());
            if provided_value.is_none() && args.silent {
                anyhow::bail!(ConversionError::MissingPlaceholderVariable {
                    var_name: slot.var_name.clone()
                })
            }
            interactive::variable(slot, provided_value)
        })?;
    let liquid_object = add_missing_provided_values(liquid_object, template_values)?;
    let (mut template_cfg, liquid_object) =
        merge_conditionals(&template_config, liquid_object, args)?;

    let all_hook_files = template_config.get_hook_files();

    execute_pre_hooks(dir, &liquid_object, &mut template_config)?;
    ignore_me::remove_unneeded_files(dir, &template_cfg.ignore, args.verbose)?;
    let mut pbar = progressbar::new();
    template::walk_dir(
        dir,
        &liquid_object,
        &mut template_cfg,
        &all_hook_files,
        &mut pbar,
    )?;
    execute_post_hooks(dir, &liquid_object, &template_config)?;
    remove_dir_files(all_hook_files, false);

    pbar.join().unwrap();

    Ok(())
}

pub(crate) fn add_missing_provided_values(
    mut liquid_object: liquid::Object,
    template_values: &HashMap<String, toml::Value>,
) -> Result<liquid::Object, anyhow::Error> {
    template_values.iter().try_for_each(|(k, v)| {
        if liquid_object.contains_key(k.as_str()) {
            return Ok(());
        }
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
        liquid_object.insert(k.clone().into(), value);
        Ok(())
    })?;
    Ok(liquid_object)
}

fn merge_conditionals(
    template_config: &Config,
    liquid_object: liquid::Object,
    args: &Args,
) -> Result<(config::TemplateConfig, liquid::Object), anyhow::Error> {
    let mut template_config = (*template_config).clone();
    let mut template_cfg = template_config.template.unwrap_or_default();
    let conditionals = template_config.conditional.take();
    if conditionals.is_none() {
        return Ok((template_cfg, liquid_object));
    }

    let mut conditionals = conditionals.unwrap();
    let mut engine = rhai::Engine::new();
    engine.on_var({
        let liqobj = liquid_object.clone();
        move |name, _, _| match liqobj.get(name) {
            Some(value) => Ok(value.as_view().as_scalar().map(|scalar| {
                scalar.to_bool().map_or_else(
                    || {
                        let v = scalar.to_kstr();
                        v.as_str().into()
                    },
                    |v| v.into(),
                )
            })),
            None => Ok(None),
        }
    });

    for (_, conditional_template_cfg) in conditionals
        .iter_mut()
        .filter(|(key, _)| engine.eval_expression::<bool>(key).unwrap_or_default())
    {
        if let Some(mut extra_includes) = conditional_template_cfg.include.take() {
            let mut includes = template_cfg.include.unwrap_or_default();
            includes.append(&mut extra_includes);
            template_cfg.include = Some(includes);
        }
        if let Some(mut extra_excludes) = conditional_template_cfg.exclude.take() {
            let mut excludes = template_cfg.exclude.unwrap_or_default();
            excludes.append(&mut extra_excludes);
            template_cfg.exclude = Some(excludes);
        }
        if let Some(mut extra_ignores) = conditional_template_cfg.ignore.take() {
            let mut ignores = template_cfg.ignore.unwrap_or_default();
            ignores.append(&mut extra_ignores);
            template_cfg.ignore = Some(ignores);
        }
        if let Some(extra_placeholders) = conditional_template_cfg.placeholders.take() {
            match template_config.placeholders.as_mut() {
                Some(placeholders) => {
                    for (k, v) in extra_placeholders.0 {
                        placeholders.0.insert(k, v);
                    }
                }
                None => {
                    template_config.placeholders = Some(extra_placeholders);
                }
            }
        }
    }

    template_config.template = Some(template_cfg);
    let template =
        project_variables::fill_project_variables(liquid_object, &template_config, |slot| {
            if args.silent {
                anyhow::bail!(ConversionError::MissingPlaceholderVariable {
                    var_name: slot.var_name.clone()
                })
            }
            interactive::variable(slot, None)
        })?;
    template_cfg = template_config.template.unwrap_or_default();

    Ok((template_cfg, template))
}

fn rename_warning(name: &ProjectName) {
    if !name.is_crate_name() {
        warn!(
            "{} `{}` {} `{}`{}",
            style("Renaming project called").bold(),
            style(&name.user_input).bold().yellow(),
            style("to").bold(),
            style(&name.kebab_case()).bold().green(),
            style("...").bold()
        );
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

        let r = auto_locate_template_dir(tmp.path(), |_slots| Err(anyhow!("test")))?;
        assert_eq!(tmp.path(), r);
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

        let r = auto_locate_template_dir(tmp.path(), |_slots| Err(anyhow!("test")))?;
        assert_eq!(tmp.path().join("dir2/dir2_2"), r);
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

        let r = auto_locate_template_dir(tmp.path(), |slots| match &slots.var_info {
            VarInfo::Bool { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let Some(mut choices) = entry.choices.clone() {
                    choices.sort();
                    let expected = vec![
                        Path::new("dir2").join("dir2_2").to_string(),
                        "dir4".to_string(),
                    ];
                    assert_eq!(expected, choices);
                    Ok("my_path".to_string())
                } else {
                    anyhow::bail!("Missing choices")
                }
            }
        });
        assert_eq!(tmp.path().join("my_path"), r?);

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

    pub fn create_file(base_path: &TempDir, path: &str, contents: &str) -> anyhow::Result<()> {
        let path = base_path.path().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::File::create(&path)?.write_all(contents.as_ref())?;
        Ok(())
    }
}
