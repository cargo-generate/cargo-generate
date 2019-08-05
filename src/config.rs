use failure::{bail, Fail, format_err};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::convert::TryFrom;
use globset::{GlobSet, GlobSetBuilder, Glob};
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub template: Template,
}

#[derive(Deserialize, Debug)]
struct Template {
    pub name: Option<String>,
    #[serde(rename = "repo")]
    pub repository: Option<String>,
    pub placeholders: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, failure::Error> {
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
            name = "cargo-template-foo"
            repo = "https://github.com/rust-lang-nursery/cargo-template-foo"
            ignore = ["**/*.ignore"]
            placeholders = ["authors", "project-name"]
            include = ["Cargo.toml"]
        "#
            .as_bytes(),
        ).unwrap();

        let config = Config::new(&config_path).unwrap();

        assert_eq!(config.template, Template {
            name: Some("cargo-template-foo".into()),
            repository: Some("https://github.com/rust-lang-nursery/cargo-template-foo".into()),
            ignore: Some(vec!["**/*.ignore".into()]),
            placeholders: Some(vec!["authors".into(), "project-name".into()]),
            include: Some(vec!["Cargo.toml".into()]),
            exclude: None
        })
    }
}
