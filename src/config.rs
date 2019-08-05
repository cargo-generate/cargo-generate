use serde::Deserialize;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use toml;

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
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, failure::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_deserializes_config() {
        let test_dir = tempdir().unwrap();
        let config_path = test_dir.path().join(".gen.toml");
        let mut file = File::create(&config_path).unwrap();

        file.write_all(
            r#"
            [template]
            include = ["Cargo.toml"]
        "#
            .as_bytes(),
        ).unwrap();

        let config = Config::new(&config_path).unwrap();

        assert_eq!(config.template, TemplateConfig {
            include: Some(vec!["Cargo.toml".into()]),
        })
    }
}
