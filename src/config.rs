use failure::{bail, Fail};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use globset::GlobSet;
use toml;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    pub template: Template,
}

struct Template {
    pub name: Option<String>,
    pub repository: Option<String>,
    pub placeholders: Option<String>,
    pub parse_matcher: Option<GlobSet>,
}

#[serde(try_from="TemplateRaw")]
impl TryFrom<TemplateRaw> for Template {
    fn from(raw_template_config: TemplateRaw) -> Result<Self, failure::Error> {
        match raw_template_config {
            TemplateRaw {
                include: Some(_),
                exclude: None,
                ..
            } => {


            },
            TemplateRaw {
                include: None,
                exclude: Some(_),
                ..
            } => {


            },
            TemplateRaw {
                include: Some(_),
                exclude: Some(_),
                ..
            } => {
                let err = TemplateConfigError::XorFieldError {
                    field_a: "include".into(),
                    field_b: "exclude".into(),
                };

                Err(err.into())
            },
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct TemplateRaw {
    pub name: Option<String>,
    #[serde(rename = "repo")]
    pub repository: Option<String>,
    pub placeholders: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Fail)]
enum TemplateConfigError {
    #[fail(
        display = "config fields {} and {} are mutually exclusive",
        field_a, field_b
    )]
    XorFieldError { field_a: String, field_b: String },
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, failure::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents)?;

        println!("{:#?}", config);


        Ok(config)
    }
}

impl Template {
    use globset::Glob;

    pub fn should_parse(&self, path: Path) -> Result<bool, failure::Error> {
        match self {
            Template {
                include: Some(includes),
                exclude: None,
                ..
            } => check_against_whitelist(path, includes),
            Template {
                exclude: Some(excludes),
                include: None,
                ..
            } => check_against_blacklist(path, excludes),
            _ => unreachable!("Precondition violated: include and exclude are mutually exclusive")
        }
    }

    fn check_against_whitelist(path: Path, includes: Vec<String>) -> Result<bool, failure::Error> {
        let mut builder = GlobSetMatcher
        includes.iter().map(Glob)
    }

    fn check_against_whitelist(path: Path, includes: Vec<String>) -> Result<bool, failure::Error> {

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

    #[test]
    fn errors_on_include_and_exclude() {
        let test_dir = tempdir().unwrap();
        let config_path = test_dir.path().join(".gen.toml");
        let mut file = File::create(&config_path).unwrap();

        file.write_all(
            r#"
            [template]
            name = "cargo-template-foo"
            repo = "https://github.com/rust-lang-nursery/cargo-template-foo"
            placeholders = ["authors, project-name"]
            ignore = ["**/*.ignore"]
            include = ["fileb.toml"]
            exclude = ["filea.toml"]
        "#
            .as_bytes(),
        ).unwrap();

        let err = Config::new(&config_path)
            .unwrap_err();
        let err: &TemplateConfigError = err
            .downcast_ref()
            .unwrap();
    
        match err {
            TemplateConfigError::XorFieldError {
                ..
            } => (),
            _ => panic!("incorrect error returned")
        };
    }
}
