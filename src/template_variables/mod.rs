mod authors;
mod crate_name;
mod crate_type;
mod os_arch;
mod project_dir;
mod project_name;
mod project_name_input;

use crate::{emoji, GenerateArgs};

use anyhow::Result;
use console::style;
use log::info;
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Display, fs, path::Path};
use toml::Value;

pub use authors::{get_authors, Authors};
pub use crate_name::CrateName;
pub use crate_type::CrateType;
pub use os_arch::get_os_arch;
pub use project_dir::ProjectDir;
pub use project_name::ProjectName;
pub use project_name_input::ProjectNameInput;

fn load_env_template_values() -> Result<HashMap<String, toml::Value>> {
    //FIXME: use this variable to be in sync with args
    let mut values = std::env::var("CARGO_GENERATE_TEMPLATE_VALUES_FILE")
        .ok()
        .map_or(Ok(Default::default()), |path| {
            read_template_values_file(Path::new(&path))
        })?;

    values.extend(std::env::vars().filter_map(|(key, value)| {
        key.strip_prefix("CARGO_GENERATE_VALUE_")
            .map(|key| (key.to_lowercase(), Value::from(value)))
    }));

    Ok(values)
}

fn load_args_template_values(args: &GenerateArgs) -> Result<HashMap<String, toml::Value>> {
    let mut values = args
        .template_values_file
        .as_ref()
        .map(Path::new)
        .map_or(Ok(Default::default()), |path| {
            read_template_values_file(path)
        })?;

    values.extend(read_template_values_from_definitions(&args.define)?);
    Ok(values)
}

pub fn load_env_and_args_template_values(
    args: &GenerateArgs,
) -> Result<HashMap<String, toml::Value>> {
    let mut template_variables = load_env_template_values()?;
    template_variables.extend(load_args_template_values(args)?);
    Ok(template_variables)
}

fn read_template_values_file(path: &Path) -> Result<HashMap<String, Value>> {
    match fs::read_to_string(path) {
        Ok(ref contents) => toml::from_str::<TemplateValuesToml>(contents)
            .map(|v| v.values)
            .map_err(|e| e.into()),
        Err(e) => anyhow::bail!(
            "{} {} \"{}\": {}",
            emoji::ERROR,
            style("Values File Error:").bold().red(),
            style(path.display()).bold().red(),
            style(e).bold().red(),
        ),
    }
}

fn read_template_values_from_definitions(
    definitions: &[impl AsRef<str> + Display],
) -> Result<HashMap<String, toml::Value>> {
    let mut values = HashMap::with_capacity(definitions.len());
    let key_value_regex = Regex::new(r"^([a-zA-Z]+[a-zA-Z0-9\-_]*)\s*=\s*(.+)$").unwrap();

    definitions
        .iter()
        .try_fold(&mut values, |template_values, definition| {
            key_value_regex.captures(definition.as_ref()).map_or_else(
                || {
                    Err(anyhow::anyhow!(
                        "{} {} {}",
                        emoji::ERROR,
                        style("Failed to parse value:").bold().red(),
                        style(definition).bold().red(),
                    ))
                },
                |cap| {
                    let key = cap.get(1).unwrap().as_str().to_string();
                    let value = cap.get(2).unwrap().as_str().to_string();

                    info!(
                        "{} {} (variable provided via CLI)",
                        emoji::WRENCH,
                        style(format!("{key}: {value:?}")).bold(),
                    );
                    template_values.insert(key, Value::from(value));
                    Ok(template_values)
                },
            )
        })?;
    Ok(values)
}

#[derive(Deserialize, Debug, PartialEq)]
struct TemplateValuesToml {
    pub(crate) values: HashMap<String, toml::Value>,
}

#[cfg(test)]
mod test {
    use super::read_template_values_from_definitions;

    #[test]
    fn names_must_start_with_word_char() {
        let definitions = vec!["0key=42"];
        let result = read_template_values_from_definitions(&definitions);
        assert!(result.is_err());

        let definitions = vec!["$key=42"];
        let result = read_template_values_from_definitions(&definitions);
        assert!(result.is_err());

        let definitions = vec!["-key=42"];
        let result = read_template_values_from_definitions(&definitions);
        assert!(result.is_err());

        let definitions = vec!["_key=42"];
        let result = read_template_values_from_definitions(&definitions);
        assert!(result.is_err());
    }

    #[test]
    fn names_may_contain_digits() {
        let definitions = vec!["my0123456789key=42"];
        let result = read_template_values_from_definitions(&definitions).unwrap();

        let val = result["my0123456789key"].as_str().unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn names_may_contain_dash() {
        let definitions = vec!["my-key=42"];
        let result = read_template_values_from_definitions(&definitions).unwrap();

        let val = result["my-key"].as_str().unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn names_may_contain_underscore() {
        let definitions = vec!["my_key=42"];
        let result = read_template_values_from_definitions(&definitions).unwrap();

        let val = result["my_key"].as_str().unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn spaces_are_not_allowed_in_names() {
        let definitions = vec!["my key=42"];
        let result = read_template_values_from_definitions(&definitions);
        assert!(result.is_err());
    }

    #[test]
    fn spaces_around_assignment_is_ok() {
        let definitions = vec!["key   =      42"];
        let result = read_template_values_from_definitions(&definitions).unwrap();

        let val = result["key"].as_str().unwrap();
        assert_eq!(val, "42");
    }
}
