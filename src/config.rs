use crate::emoji;
use anyhow::Result;
use semver::VersionReq;
use serde::Deserialize;
use std::path::Path;
use std::{collections::HashMap, fs};
use std::{convert::TryFrom, io::ErrorKind};

pub(crate) const CONFIG_FILE_NAME: &str = "cargo-generate.toml";

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct Config {
    pub(crate) template: Option<TemplateConfig>,
    pub(crate) placeholders: Option<TemplateSlotsTable>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct ConfigValues {
    pub(crate) values: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct TemplateConfig {
    pub(crate) cargo_generate_version: Option<VersionReq>,
    pub(crate) include: Option<Vec<String>>,
    pub(crate) exclude: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct TemplateSlotsTable(pub(crate) HashMap<String, toml::Value>);

impl TryFrom<String> for Config {
    type Error = toml::de::Error;

    fn try_from(contents: String) -> Result<Self, Self::Error> {
        let mut config: Config = toml::from_str(&contents)?;

        if let Some(ref mut template) = config.template {
            if template.include.is_some() && template.exclude.is_some() {
                template.exclude = None;
                println!(
                    "{0} Your {1} contains both an include and exclude list. \
                        Only the include list will be considered. \
                        You should remove the exclude list for clarity. {0}",
                    emoji::WARN,
                    CONFIG_FILE_NAME
                )
            }
        }

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
                Ok(contents) => Config::try_from(contents)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::tempdir;
    use toml::Value;

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
                exclude: None
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
                placeholders: None
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
