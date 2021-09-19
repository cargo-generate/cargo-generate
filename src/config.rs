use anyhow::Result;
use semver::VersionReq;
use serde::Deserialize;
use std::path::Path;
use std::{collections::HashMap, fs};
use std::{convert::TryFrom, io::ErrorKind};
use walkdir::WalkDir;

pub const CONFIG_FILE_NAME: &str = "cargo-generate.toml";

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub struct Config {
    pub template: Option<TemplateConfig>,
    pub placeholders: Option<TemplateSlotsTable>,
    pub hooks: Option<HooksConfig>,
    pub conditional: Option<HashMap<String, ConditionalConfig>>,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub struct HooksConfig {
    pub pre: Option<Vec<String>>,
    pub post: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub struct TemplateConfig {
    pub cargo_generate_version: Option<VersionReq>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ConditionalConfig {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub placeholders: Option<TemplateSlotsTable>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TemplateSlotsTable(pub HashMap<String, toml::Value>);

impl TryFrom<String> for Config {
    type Error = toml::de::Error;

    fn try_from(contents: String) -> Result<Self, Self::Error> {
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl Config {
    pub(crate) fn from_path<P>(path: &Option<P>) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        match path {
            Some(path) => match fs::read_to_string(path) {
                Ok(contents) => Self::try_from(contents)
                    .map(Option::from)
                    .map_err(|e| e.into()),
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => Ok(None),
                    _ => anyhow::bail!(e),
                },
            },
            None => Ok(None),
        }
    }

    pub fn get_pre_hooks(&self) -> Vec<String> {
        self.hooks
            .as_ref()
            .map(|h| h.pre.as_ref().map(Clone::clone).unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_post_hooks(&self) -> Vec<String> {
        self.hooks
            .as_ref()
            .map(|h| h.post.as_ref().map(Clone::clone).unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_hook_files(&self) -> Vec<String> {
        let mut pre = self.get_pre_hooks();
        pre.append(&mut self.get_post_hooks());
        pre
    }
}

pub fn locate_template_configs(dir: &Path) -> Result<Vec<String>> {
    let mut result = vec![];

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_name() == CONFIG_FILE_NAME {
            let path = entry
                .path()
                .parent()
                .unwrap()
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            result.push(path)
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::tests::{create_file, PathString};

    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::tempdir;
    use toml::Value;

    #[test]
    fn locate_configs_returns_empty_upon_failure() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;

        let result = locate_template_configs(tmp.path())?;
        assert_eq!(Vec::new() as Vec<String>, result);
        Ok(())
    }

    #[test]
    fn locate_configs_can_locate_paths_with_cargo_generate() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        create_file(&tmp, "dir1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_1/Cargo.toml", "")?;
        create_file(&tmp, "dir2/dir2_2/cargo-generate.toml", "")?;
        create_file(&tmp, "dir3/Cargo.toml", "")?;
        create_file(&tmp, "dir4/cargo-generate.toml", "")?;

        let expected = vec![
            Path::new("dir2").join("dir2_2").to_string(),
            "dir4".to_string(),
        ];
        let result = {
            let mut x = locate_template_configs(tmp.path())?;
            x.sort();
            x
        };
        assert_eq!(expected, result);
        Ok(())
    }

    #[test]
    fn test_deserializes_config() {
        let test_dir = tempdir().unwrap();
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

        let config = Config::from_path(&Some(config_path)).unwrap().unwrap();

        assert_eq!(
            config.template,
            Some(TemplateConfig {
                cargo_generate_version: Some(VersionReq::from_str(">=0.8.0").unwrap()),
                include: Some(vec!["Cargo.toml".into()]),
                exclude: None,
                ignore: None,
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
