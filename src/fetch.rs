//! Fetching a template: get it into a temp directory, then work out
//! which directory inside it is actually the template.
//!
//! Both halves are one step from the caller's view — `generate()` asks
//! for a local template and gets back a temp dir it owns, the resolved
//! template directory, and the branch that was checked out.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use console::style;
use fs_err as fs;
use tempfile::TempDir;

use crate::config::{locate_template_configs, Config, CONFIG_FILE_NAME};
use crate::copy::copy_files_recursively;
use crate::emoji;
use crate::git::{self, try_get_branch_from_path};
use crate::interactive::prompt_and_check_variable;
use crate::project_variables::{Choice, StringEntry, StringKind, TemplateSlots, VarInfo};
use crate::read_default_variable_value_from_template;
use crate::user_parsed_input::{Source, UserParsedInput};
use crate::utils::tmp_dir;

/// A template materialized into a temp directory.
///
/// Owns the temp directory: dropping a `FetchedSource` deletes the
/// materialized template, so it has to outlive every path derived
/// from it.
#[derive(Debug)]
pub struct FetchedSource {
    root: TempDir,
    template_dir: PathBuf,
    branch: Option<String>,
}

impl FetchedSource {
    /// A freshly materialized source, before sub-template resolution.
    /// `template_dir` starts at the root and [`Self::narrow_to`]
    /// moves it inward.
    pub(crate) fn new(root: TempDir, branch: Option<String>) -> Self {
        let template_dir = root.path().to_owned();
        Self {
            root,
            template_dir,
            branch,
        }
    }

    /// Point at the sub-directory that is the actual template.
    fn narrow_to(mut self, template_dir: PathBuf) -> Self {
        self.template_dir = template_dir;
        self
    }

    /// The temp directory holding the whole materialized source.
    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// The directory the template itself lives in — the root, or a
    /// sub-template below it.
    pub fn template_dir(&self) -> &Path {
        &self.template_dir
    }

    /// The branch the source was on, when it could be determined.
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

pub fn prepare_local_template(source_template: &UserParsedInput) -> Result<FetchedSource> {
    let fetched = get_source_template_into_temp(source_template.location())?;
    let template_folder = resolve_template_dir(
        fetched.root(),
        source_template.subfolder(),
        source_template.silent(),
    )?;

    Ok(fetched.narrow_to(template_folder))
}

fn get_source_template_into_temp(source: &Source) -> Result<FetchedSource> {
    let fetched = match source {
        #[cfg(feature = "git")]
        Source::Git(git) => {
            let fetched = git::clone_git_template_into_temp(
                git.url(),
                git.branch(),
                git.tag(),
                git.revision(),
                git.identity(),
                git.gitconfig(),
                git.skip_submodules,
            )?;
            git::remove_history(fetched.root())?;
            fetched
        }
        Source::Local(path) => {
            let root = tmp_dir()?;
            copy_files_recursively(path, root.path(), false)?;
            git::remove_history(root.path())?;
            FetchedSource::new(root, try_get_branch_from_path(path))
        }
    };

    // Both arms materialize verbatim; the one liquid pass runs here so
    // a local folder and a clone of the same template cannot diverge.
    strip_liquid_suffixes(fetched.root())?;

    Ok(fetched)
}

/// The suffix marking a file that should be rendered rather than copied
/// verbatim: `README.md.liquid` becomes `README.md`.
const LIQUID_SUFFIX: &str = ".liquid";

/// Resolve `.liquid` filenames in a materialized template.
///
/// `foo.liquid` is renamed to `foo`. When a template ships both, the
/// `.liquid` file wins — the rename replaces its plain twin, which is
/// how a template overrides a file it also ships unrendered.
///
/// Runs for every source. Fetching used to do this two different ways
/// — the local copy resolved suffixes inline while cloning resolved
/// them afterwards — which is one rule with two implementations and
/// two chances to drift. Materializing is now plain for both, and this
/// is the single place the rule lives.
fn strip_liquid_suffixes(dir: impl AsRef<Path>) -> Result<()> {
    for entry in fs::read_dir(dir.as_ref())? {
        let entry = entry?;
        let entry_type = entry.file_type()?;

        if entry_type.is_dir() {
            strip_liquid_suffixes(entry.path())?;
        } else if entry_type.is_file() {
            let path = entry.path().to_string_lossy().to_string();
            if let Some(new_path) = path.clone().strip_suffix(LIQUID_SUFFIX) {
                fs::rename(path, new_path)?;
            }
        }
    }
    Ok(())
}

/// resolve the template location for the actual template to expand
fn resolve_template_dir(
    template_base_dir: &Path,
    subfolder: Option<&str>,
    silent: bool,
) -> Result<PathBuf> {
    let template_dir = resolve_template_dir_subfolder(template_base_dir, subfolder)?;
    auto_locate_template_dir(template_dir, &mut |slots| {
        select_sub_template(slots, silent, |slots| {
            prompt_and_check_variable(slots, None)
        })
    })
}

fn select_sub_template(
    slots: &TemplateSlots,
    silent: bool,
    prompt: impl FnOnce(&TemplateSlots) -> Result<String>,
) -> Result<String> {
    if !silent {
        return prompt(slots);
    }

    read_default_variable_value_from_template(slots).map_err(|()| {
        anyhow!(
            "{} {}",
            emoji::ERROR,
            style(format!(
                "Option `--silent` provided, but `{}` has no default value.",
                slots.var_name
            ))
            .bold()
            .red()
        )
    })
}

/// join the base-dir and the subfolder, ensuring that we stay within the template directory
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
                        kind: StringKind::Choices(
                            config_paths
                                .into_iter()
                                .map(|p| Choice::new(p.display().to_string()))
                                .collect(),
                        ),
                        regex: None,
                    }),
                },
            };
            let path = prompt(&prompt_args)?;

            // recursively retry to resolve the template,
            // until we hit a single or no config, identifying the final template folder
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
                            kind: StringKind::Choices(
                                sub_templates.iter().cloned().map(Choice::new).collect(),
                            ),
                            regex: None,
                        }),
                    },
                };
                let path = prompt(&prompt_args)?;

                // recursively retry to resolve the template,
                // until we hit a single or no config, identifying the final template folder
                auto_locate_template_dir(
                    resolve_template_dir_subfolder(config_path, Some(path))?,
                    prompt,
                )
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_variables::{StringEntry, StringKind, TemplateSlots, VarInfo};
    use crate::utils::tmp_dir;
    use anyhow::anyhow;
    use std::fs;
    use std::io::Write;

    /// `Path::display().to_string()` without the ceremony at each call site.
    trait PathString {
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

    fn create_file(
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

    #[test]
    fn strip_liquid_suffixes_renames_and_lets_liquid_win() {
        let tmp = TempDir::new().unwrap();
        create_file(&tmp, "plain.md", "plain only").unwrap();
        create_file(&tmp, "rendered.md.liquid", "rendered only").unwrap();
        // a template shipping both: the .liquid file overrides its twin
        create_file(&tmp, "both.md", "the plain twin").unwrap();
        create_file(&tmp, "both.md.liquid", "the liquid one").unwrap();
        create_file(&tmp, "nested/deep.rs.liquid", "nested").unwrap();

        strip_liquid_suffixes(tmp.path()).unwrap();

        let read = |p: &str| fs::read_to_string(tmp.path().join(p)).unwrap();
        assert_eq!(read("plain.md"), "plain only");
        assert_eq!(read("rendered.md"), "rendered only");
        assert_eq!(read("both.md"), "the liquid one", "the .liquid file wins");
        assert_eq!(read("nested/deep.rs"), "nested", "recurses");

        for gone in [
            "rendered.md.liquid",
            "both.md.liquid",
            "nested/deep.rs.liquid",
        ] {
            assert!(!tmp.path().join(gone).exists(), "{gone} should be renamed");
        }
    }

    #[test]
    fn auto_locate_template_returns_base_when_no_cargo_generate_is_found() -> anyhow::Result<()> {
        let tmp = tmp_dir().unwrap();
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
        let tmp = tmp_dir().unwrap();
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
        let tmp = tmp_dir().unwrap();
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
            VarInfo::Bool { .. } | VarInfo::Array { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let StringKind::Choices(choices) = entry.kind.clone() {
                    let expected = vec!["sub1".to_string(), "sub2".to_string()];
                    assert_eq!(
                        expected,
                        choices.iter().map(|c| c.value.clone()).collect::<Vec<_>>()
                    );
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
    fn resolve_template_dir_uses_default_subtemplate_in_silent_mode() -> anyhow::Result<()> {
        let tmp = tmp_dir().unwrap();
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

        let actual = resolve_template_dir(tmp.path(), None, true)?.canonicalize()?;
        let expected = tmp.path().join("sub1").canonicalize()?;

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn select_sub_template_uses_configured_default_in_silent_mode() -> anyhow::Result<()> {
        let slot = sub_template_slot(Some("sub1"));

        let actual = select_sub_template(&slot, true, |_| {
            unreachable!("silent mode should not prompt")
        })?;

        assert_eq!("sub1", actual);
        Ok(())
    }

    #[test]
    fn select_sub_template_errors_without_default_in_silent_mode() {
        let slot = sub_template_slot(None);

        let err = select_sub_template(&slot, true, |_| Ok("sub2".into())).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("Option `--silent` provided"));
        assert!(message.contains("`Template` has no default value"));
    }

    #[test]
    fn select_sub_template_prompts_outside_silent_mode() -> anyhow::Result<()> {
        let slot = sub_template_slot(Some("sub1"));

        let actual = select_sub_template(&slot, false, |slots| {
            assert_eq!("Template", slots.var_name);
            Ok("sub2".into())
        })?;

        assert_eq!("sub2", actual);
        Ok(())
    }

    fn sub_template_slot(default: Option<&str>) -> TemplateSlots {
        TemplateSlots {
            prompt: "Which sub-template should be expanded?".into(),
            var_name: "Template".into(),
            var_info: VarInfo::String {
                entry: Box::new(StringEntry {
                    default: default.map(str::to_owned),
                    kind: StringKind::Choices(vec![Choice::new("sub1"), Choice::new("sub2")]),
                    regex: None,
                }),
            },
        }
    }

    #[test]
    fn auto_locate_template_recurses_to_resolve_subtemplates() -> anyhow::Result<()> {
        let tmp = tmp_dir().unwrap();
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
            VarInfo::Bool { .. } | VarInfo::Array { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let StringKind::Choices(choices) = entry.kind.clone() {
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
                        .for_each(|(a, b)| assert_eq!(a, b.value));
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
        let tmp = tmp_dir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_2/cargo-generate.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;
        create_file(&tmp, "dir4/cargo-generate.toml", "")?;

        let actual = auto_locate_template_dir(tmp.path().to_path_buf(), &mut |slots| match &slots
            .var_info
        {
            VarInfo::Bool { .. } | VarInfo::Array { .. } => anyhow::bail!("Wrong prompt type"),
            VarInfo::String { entry } => {
                if let StringKind::Choices(choices) = entry.kind.clone() {
                    let expected = vec![
                        Path::new("dir2").join("dir2_2").to_string(),
                        "dir4".to_string(),
                    ];
                    assert_eq!(
                        expected,
                        choices.iter().map(|c| c.value.clone()).collect::<Vec<_>>()
                    );
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
}
