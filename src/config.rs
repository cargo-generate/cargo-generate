use anyhow::Result;
use indexmap::IndexMap;
use semver::VersionReq;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};
use std::{convert::TryFrom, io::ErrorKind};

use crate::Vcs;

pub const CONFIG_FILE_NAME: &str = "cargo-generate.toml";

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub struct Config {
    pub template: Option<TemplateConfig>,
    pub placeholders: Option<TemplateSlotsTable>,
    pub hooks: Option<HooksConfig>,
    pub conditional: Option<HashMap<String, ConditionalConfig>>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct HooksConfig {
    pub init: Option<Vec<String>>,
    pub pre: Option<Vec<String>>,
    pub post: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct TemplateConfig {
    pub sub_templates: Option<Vec<String>>,

    pub cargo_generate_version: Option<VersionReq>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub vcs: Option<Vcs>,
    pub init: Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ConditionalConfig {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub placeholders: Option<TemplateSlotsTable>,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TemplateSlotsTable(pub IndexMap<String, toml::Value>);

impl TryFrom<String> for Config {
    type Error = toml::de::Error;

    fn try_from(contents: String) -> Result<Self, Self::Error> {
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl Config {
    pub(crate) fn from_path(path: &Option<impl AsRef<Path>>) -> Result<Self> {
        let mut config = match path {
            Some(path) => match fs::read_to_string(path) {
                Ok(contents) => Self::try_from(contents)?,
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => Self::default(),
                    _ => anyhow::bail!(e),
                },
            },
            None => Self::default(),
        };
        config.template.get_or_insert(Default::default());
        Ok(config)
    }

    pub fn get_init_hooks(&self) -> Vec<String> {
        self.hooks
            .as_ref()
            .map(|h| h.init.clone().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_pre_hooks(&self) -> Vec<String> {
        self.hooks
            .as_ref()
            .map(|h| h.pre.clone().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_post_hooks(&self) -> Vec<String> {
        self.hooks
            .as_ref()
            .map(|h| h.post.clone().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_hook_files(&self) -> Vec<String> {
        let mut pre = self.get_init_hooks();
        pre.append(&mut self.get_pre_hooks());
        pre.append(&mut self.get_post_hooks());
        pre
    }
}

/// Search through a folder structure for template configuration files, but look no deeper than
/// a found file!
pub fn locate_template_configs(base_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::with_capacity(1);

    if base_dir.is_dir() {
        let mut paths_to_search_in = vec![base_dir.to_path_buf()];
        'next_path: while let Some(path) = paths_to_search_in.pop() {
            let mut sub_paths = vec![];
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    sub_paths.push(entry_path);
                } else if entry.file_name() == CONFIG_FILE_NAME {
                    results.push(path.strip_prefix(base_dir)?.to_path_buf());
                    continue 'next_path;
                }
            }
            paths_to_search_in.append(&mut sub_paths);
        }
    } else {
        results.push(base_dir.to_path_buf());
    }

    results.sort();
    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::tests::create_file;
    use crate::tmp_dir;

    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use toml::Value;

    #[test]
    fn locate_configs_returns_empty_upon_failure() -> anyhow::Result<()> {
        let tmp = tmp_dir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;

        let result = locate_template_configs(tmp.path())?;
        assert_eq!(Vec::new() as Vec<PathBuf>, result);
        Ok(())
    }

    #[test]
    fn locate_configs_can_locate_paths_with_cargo_generate() -> anyhow::Result<()> {
        let tmp = tmp_dir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_2/cargo-generate.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;
        create_file(&tmp, "dir4/cargo-generate.toml", "")?;

        let expected = vec![Path::new("dir2").join("dir2_2"), PathBuf::from("dir4")];
        let result = locate_template_configs(tmp.path())?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[test]
    fn locate_configs_doesnt_look_past_cargo_generate() -> anyhow::Result<()> {
        let tmp = tmp_dir().unwrap();
        create_file(&tmp, "dir1/cargo-generate.toml", "")?;
        create_file(&tmp, "dir1/dir2/cargo-generate.toml", "")?;

        let expected = vec![PathBuf::from("dir1")];
        let result = locate_template_configs(tmp.path())?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[test]
    fn test_deserializes_config() {
        let test_dir = tmp_dir().unwrap();
        let config_path = test_dir.path().join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_path).unwrap();

        file.write_all(
            r#"
            [template]
            cargo_generate_version = ">=0.8.0"
            include = ["Cargo.toml"]
            [placeholders]
            test = {a = "a"}
        "#
            .as_bytes(),
        )
        .unwrap();

        let config = Config::from_path(&Some(config_path)).unwrap();

        assert_eq!(
            config.template,
            Some(TemplateConfig {
                sub_templates: None,
                cargo_generate_version: Some(VersionReq::from_str(">=0.8.0").unwrap()),
                include: Some(vec!["Cargo.toml".into()]),
                exclude: None,
                ignore: None,
                vcs: None,
                init: None,
            })
        );
        assert!(config.placeholders.is_some());
    }

    #[test]
    fn config_try_from_handles_empty() {
        let result = Config::try_from("".to_string());
        assert!(result.is_ok(), "Config should have parsed");
        let result = result.unwrap();
        assert_eq!(
            result,
            Config {
                template: None,
                hooks: None,
                placeholders: None,
                conditional: Default::default(),
            }
        )
    }

    #[test]
    fn config_try_from_errors_on_invalid_keys() {
        let result = Config::try_from(
            r#"
            [placeholders]
            a key = { type = "bool", prompt = "foo"}
            b = { type = "string", prompt = "bar" }
            "#
            .to_string(),
        );

        assert!(result.is_err(), "Config should not have parsed");
    }

    #[test]
    fn config_try_from_handles_placeholders() {
        let result = Config::try_from(
            r#"
            [placeholders]
            a = { type = "bool", prompt = "foo", default = false }
            b = { type = "string", prompt = "bar" }
            "#
            .to_string(),
        );

        assert!(result.is_ok(), "Config should have parsed");
        let result = result.unwrap();

        assert!(
            result.placeholders.is_some(),
            "placeholders should have been filled"
        );
        let placeholders = result.placeholders.unwrap();

        assert_eq!(placeholders.0.len(), 2);

        let a = placeholders.0.get("a");
        let b = placeholders.0.get("b");

        assert!(
            a.is_some() && b.is_some(),
            "placeholder keys should have been parsed"
        );

        let a_table = a.unwrap().as_table();
        let b_table = b.unwrap().as_table();

        assert!(
            a_table.is_some() && b_table.is_some(),
            "values should have been parsed as tables"
        );

        let a_table = a_table.unwrap();
        let b_table = b_table.unwrap();

        assert_eq!(a_table.len(), 3);
        assert_eq!(b_table.len(), 2);

        let (a_type, a_prompt, a_default) = (
            a_table.get("type"),
            a_table.get("prompt"),
            a_table.get("default"),
        );
        let (b_type, b_prompt) = (b_table.get("type"), b_table.get("prompt"));

        assert_eq!(a_type, Some(&Value::String("bool".to_string())));
        assert_eq!(a_prompt, Some(&Value::String("foo".to_string())));
        assert_eq!(a_default, Some(&Value::Boolean(false)));

        assert_eq!(b_type, Some(&Value::String("string".to_string())));
        assert_eq!(b_prompt, Some(&Value::String("bar".to_string())));
    }
}
