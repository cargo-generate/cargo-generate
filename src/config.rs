use crate::emoji;
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(crate) const CONFIG_FILE_NAME: &str = "cargo-generate.toml";

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) template: TemplateConfig,
}

#[derive(Deserialize, Debug)]
pub(crate) struct TemplateConfig {
    pub(crate) include: Option<Vec<String>>,
    pub(crate) exclude: Option<Vec<String>>,
}

impl Config {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Option<Self>> {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let mut config: Config = toml::from_str(&contents)?;

                if config.template.include.is_some() && config.template.exclude.is_some() {
                    config.template.exclude = None;
                    println!(
                        "{0} Your {1} contains both an include and exclude list. \
                        Only the include list will be considered. \
                        You should remove the exclude list for clarity. {0}",
                        emoji::WARN,
                        CONFIG_FILE_NAME
                    )
                }

                Ok(Some(config))
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => anyhow::bail!(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_deserializes_config() {
        let test_dir = tempdir().unwrap();
        let config_path = test_dir.path().join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_path).unwrap();

        file.write_all(
            r#"
            [template]
            include = ["Cargo.toml"]
        "#
            .as_bytes(),
        )
        .unwrap();

        let config = Config::new(&config_path).unwrap().unwrap();

        assert_eq!(config.template.include, Some(vec!["Cargo.toml".into()]))
    }
}
