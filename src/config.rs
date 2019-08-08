use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use toml;

pub const CONFIG_FILE_NAME: &str = "cargo-generate.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub template: TemplateConfig,
}

#[derive(Deserialize, Debug)]
pub struct TemplateConfig {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Option<Self>, failure::Error> {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let config: Config = toml::from_str(&contents)?;

                Ok(Some(config))
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => failure::bail!(e),
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
