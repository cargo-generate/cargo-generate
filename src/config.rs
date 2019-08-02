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
#[serde(try_from = "TemplateRaw")]
pub struct Template {
    pub name: Option<String>,
    pub repository: Option<String>,
    pub placeholders: Option<Vec<String>>,
    pub parse_matcher: Option<GlobSet>,
}

impl TryFrom<TemplateRaw> for Template {
    type Error = failure::Error;
    fn try_from(raw_template_config: TemplateRaw) -> Result<Self, Self::Error> {
        match raw_template_config {
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
            TemplateRaw {
                include: Some(whitelist_globs),
                exclude: None,
                name, repository, placeholders,
            } => {
                let globs: Result<Vec<Glob>, globset::Error> = whitelist_globs.into_iter()
                    .map(|globstr| Glob::new(&globstr))
                    .collect();

                globs.and_then(|globs| {
                    let mut builder = GlobSetBuilder::new();

                    for glob in globs {
                        builder.add(glob);
                    }

                    Ok(builder.build())
                }).and_then(|globset| {
                    Ok(Template {
                        name, repository, placeholders,
                        parse_matcher: Some(globset?)
                    })
                }).map_err(|globsetErr| format_err!("globs borked"))
            },
            TemplateRaw {
                include: None,
                exclude: Some(blacklist_globs),
                name, repository, placeholders,
            } => {
                let globs: Result<Vec<Glob>, globset::Error> = blacklist_globs.into_iter()
                    .map(|globstr| Glob::new(&globstr))
                    .collect();

                globs.and_then(|globs| {
                    let mut builder = GlobSetBuilder::new();

                    for glob in globs {
                        builder.add(glob);
                    }

                    Ok(builder.build())
                }).and_then(|globset| {
                    Ok(Template {
                        name, repository, placeholders,
                        parse_matcher: Some(globset?)
                    })
                }).map_err(|globsetErr| format_err!("globs borked"))
            },
            TemplateRaw {
                include: None,
                exclude: None,
                name, repository, placeholders,
            } => {
                Ok(Template {
                    name, repository, placeholders,
                    parse_matcher: None
                })
            },
        }
    }
}

#[derive(Deserialize, Debug)]
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
