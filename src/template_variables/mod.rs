mod authors;
mod crate_type;
mod os_arch;
mod project_name;

use crate::{config::ConfigValues, emoji, Args};

use anyhow::Result;
use console::style;
use regex::Regex;
use std::{collections::HashMap, fmt::Display, fs, path::Path};
use toml::Value;

pub(crate) use authors::{get_authors, Authors};
pub(crate) use crate_type::CrateType;
pub(crate) use os_arch::get_os_arch;
pub(crate) use project_name::ProjectName;

pub(crate) fn resolve_template_values(args: &Args) -> Result<HashMap<String, Value>> {
    let mut values = std::env::var("CARGO_GENERATE_TEMPLATE_VALUES_FILE")
        .ok()
        .map_or(Ok(Default::default()), |path| {
            read_template_values_file(Path::new(&path))
        })?;

    values.extend(std::env::vars().filter_map(|(key, value)| {
        key.strip_prefix("CARGO_GENERATE_VALUE_")
            .map(|key| (key.to_lowercase(), Value::from(value)))
    }));

    values.extend(
        args.template_values_file
            .as_ref()
            .map(|p| Path::new(p))
            .map_or(Ok(Default::default()), |path| {
                read_template_values_file(path)
            })?,
    );

    add_cli_defined_values(&mut values, &args.define)?;

    Ok(values)
}

fn read_template_values_file(path: &Path) -> Result<HashMap<String, Value>> {
    match fs::read_to_string(path) {
        Ok(ref contents) => toml::from_str::<ConfigValues>(contents)
            .map(|v| v.values)
            .map_err(|e| e.into()),
        Err(e) => anyhow::bail!(
            "{} {} {}",
            emoji::ERROR,
            style("Values File Error:").bold().red(),
            style(e).bold().red(),
        ),
    }
}

fn add_cli_defined_values<S: AsRef<str> + Display>(
    template_values: &mut HashMap<String, Value>,
    definitions: &[S],
) -> Result<()> {
    let key_value_regex = Regex::new(r"^([a-zA-Z]+[a-zA-Z0-9\-_]*)\s*=\s*(.+)$").unwrap();

    definitions
        .iter()
        .try_fold(
            template_values,
            |template_values, definition| match key_value_regex.captures(definition.as_ref()) {
                Some(cap) => {
                    let key = cap.get(1).unwrap().as_str().to_string();
                    let value = cap.get(2).unwrap().as_str().to_string();
                    println!("{} => '{}'", key, value);
                    template_values.insert(key, Value::from(value));
                    Ok(template_values)
                }
                None => Err(anyhow::anyhow!(
                    "{} {} {}",
                    emoji::ERROR,
                    style("Failed to parse value:").bold().red(),
                    style(definition).bold().red(),
                )),
            },
        )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::add_cli_defined_values;
    use std::collections::HashMap;

    #[test]
    fn names_must_start_with_word_char() {
        let mut template_values = HashMap::new();
        let definitions = vec!["0key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_err());

        let definitions = vec!["$key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_err());

        let definitions = vec!["-key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_err());

        let definitions = vec!["_key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_err());
    }

    #[test]
    fn names_may_contain_digits() {
        let mut template_values = HashMap::new();
        let definitions = vec!["my0123456789key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_ok());

        let val = template_values
            .get("my0123456789key")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn names_may_contain_dash() {
        let mut template_values = HashMap::new();
        let definitions = vec!["my-key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_ok());

        let val = template_values.get("my-key").unwrap().as_str().unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn names_may_contain_underscore() {
        let mut template_values = HashMap::new();
        let definitions = vec!["my_key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_ok());

        let val = template_values.get("my_key").unwrap().as_str().unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn spaces_are_not_allowed_in_names() {
        let mut template_values = HashMap::new();
        let definitions = vec!["my key=42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_err());
    }

    #[test]
    fn spaces_around_assignment_is_ok() {
        let mut template_values = HashMap::new();
        let definitions = vec!["key   =      42"];
        let result = add_cli_defined_values(&mut template_values, &definitions);
        assert!(result.is_ok());

        let val = template_values.get("key").unwrap().as_str().unwrap();
        assert_eq!(val, "42");
    }
}
