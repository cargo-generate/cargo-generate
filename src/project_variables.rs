use anyhow::Result;
use indexmap::IndexMap;
use liquid_core::model::map::Entry;
use liquid_core::Value;
use regex::Regex;
use thiserror::Error;

use crate::{
    config::{Config, TemplateSlotsTable},
    template::LiquidObjectResource,
};

#[derive(Debug)]
pub struct TemplateSlots {
    pub(crate) var_name: String,
    pub(crate) var_info: VarInfo,
    pub(crate) prompt: String,
}

// Information needed to prompt for a typed value
#[derive(Debug, Clone)]
pub enum VarInfo {
    Bool { default: Option<bool> },
    String { entry: Box<StringEntry> },
}

#[derive(Debug, Clone)]
pub struct StringEntry {
    pub(crate) default: Option<String>,
    pub(crate) choices: Option<Vec<String>>,
    pub(crate) regex: Option<Regex>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ConversionError {
    #[error("parameter `{parameter}` of placeholder `{var_name}` should be a `{correct_type}`")]
    WrongTypeParameter {
        var_name: String,
        parameter: String,
        correct_type: String,
    },
    #[error("placeholder `{var_name}` should be a table")]
    InvalidPlaceholderFormat { var_name: String },
    #[error("missing prompt question for `{var_name}`")]
    MissingPrompt { var_name: String },
    #[error("choices array empty for `{var_name}`")]
    EmptyChoices { var_name: String },
    #[error("default is `{default}`, but is not a valid value in choices array `{choices:?}` for  `{var_name}`")]
    InvalidDefault {
        var_name: String,
        default: String,
        choices: Vec<String>,
    },
    #[error(
        "invalid type for variable `{var_name}`: `{value}` possible values are `bool` and `string`"
    )]
    InvalidVariableType { var_name: String, value: String },
    #[error("bool type does not support `choices` field")]
    ChoicesOnBool { var_name: String },
    #[error("bool type does not support `regex` field")]
    RegexOnBool { var_name: String },
    #[error("variable `{var_name}` was missing in config file running on silent mode")]
    MissingPlaceholderVariable { var_name: String },
    #[error("field `{field}` of variable `{var_name}` does not match configured regex")]
    RegexDoesntMatchField { var_name: String, field: String },
    #[error("regex of `{var_name}` is not a valid regex. {error}")]
    InvalidRegex {
        var_name: String,
        regex: String,
        error: regex::Error,
    },
    #[error("placeholder `{var_name}` is not valid as you can't override `project-name`, `crate_name`, `crate_type`, `authors` and `os-arch`")]
    InvalidPlaceholderName { var_name: String },
}

#[derive(Debug, Clone, PartialEq)]
enum SupportedVarValue {
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SupportedVarType {
    Bool,
    String,
}

const RESERVED_NAMES: [&str; 7] = [
    "authors",
    "os-arch",
    "project-name",
    "crate_name",
    "crate_type",
    "within_cargo_project",
    "is_init",
];

/// For each defined placeholder, try to add it with value as a variable to the template_object.
pub fn fill_project_variables(
    template_object: &LiquidObjectResource,
    config: &Config,
    value_provider: impl Fn(&TemplateSlots) -> Result<Value>,
) -> Result<()> {
    let template_slots = config
        .placeholders
        .as_ref()
        .map(try_into_template_slots)
        .unwrap_or_else(|| Ok(IndexMap::new()))?;

    for (&key, slot) in template_slots.iter() {
        match template_object
            .lock()
            .unwrap()
            .borrow_mut()
            .entry(key.to_string())
        {
            Entry::Occupied(_) => {
                // we already have the value from the config file
            }
            Entry::Vacant(entry) => {
                // we don't have the file from the config but we can ask for it
                let value = value_provider(slot)?;
                entry.insert(value);
            }
        }
    }
    Ok(())
}

fn try_into_template_slots(
    TemplateSlotsTable(table): &TemplateSlotsTable,
) -> Result<IndexMap<&str, TemplateSlots>, ConversionError> {
    let mut slots = IndexMap::with_capacity(table.len());
    for (key, values) in table.iter() {
        slots.insert(key.as_str(), try_key_value_into_slot(key, values)?);
    }
    Ok(slots)
}

fn try_key_value_into_slot(
    key: &str,
    values: &toml::Value,
) -> Result<TemplateSlots, ConversionError> {
    if RESERVED_NAMES.contains(&key) {
        return Err(ConversionError::InvalidPlaceholderName {
            var_name: key.to_string(),
        });
    }

    let table = values
        .as_table()
        .ok_or(ConversionError::InvalidPlaceholderFormat {
            var_name: key.to_string(),
        })?;

    let var_type = extract_type(key, table.get("type"))?;
    let regex = extract_regex(key, var_type, table.get("regex"))?;
    let prompt = extract_prompt(key, table.get("prompt"))?;
    let choices = extract_choices(key, var_type, regex.as_ref(), table.get("choices"))?;
    let default_choice = extract_default(
        key,
        var_type,
        regex.as_ref(),
        table.get("default"),
        choices.as_ref(),
    )?;

    let var_info = match (var_type, default_choice) {
        (SupportedVarType::Bool, Some(SupportedVarValue::Bool(value))) => VarInfo::Bool {
            default: Some(value),
        },
        (SupportedVarType::String, Some(SupportedVarValue::String(value))) => VarInfo::String {
            entry: Box::new(StringEntry {
                default: Some(value),
                choices,
                regex,
            }),
        },
        (SupportedVarType::Bool, None) => VarInfo::Bool { default: None },
        (SupportedVarType::String, None) => VarInfo::String {
            entry: Box::new(StringEntry {
                default: None,
                choices,
                regex,
            }),
        },
        _ => unreachable!("It should not have come to this..."),
    };
    Ok(TemplateSlots {
        var_name: key.to_string(),
        var_info,
        prompt,
    })
}

fn extract_regex(
    var_name: &str,
    var_type: SupportedVarType,
    table_entry: Option<&toml::Value>,
) -> Result<Option<Regex>, ConversionError> {
    match (var_type, table_entry) {
        (SupportedVarType::Bool, Some(_)) => Err(ConversionError::RegexOnBool {
            var_name: var_name.into(),
        }),
        (SupportedVarType::String, Some(toml::Value::String(value))) => match Regex::new(value) {
            Ok(regex) => Ok(Some(regex)),
            Err(e) => Err(ConversionError::InvalidRegex {
                var_name: var_name.into(),
                regex: value.clone(),
                error: e,
            }),
        },
        (SupportedVarType::String, Some(_)) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "regex".to_string(),
            correct_type: "String".to_string(),
        }),
        (_, None) => Ok(None),
    }
}

fn extract_type(
    var_name: &str,
    table_entry: Option<&toml::Value>,
) -> Result<SupportedVarType, ConversionError> {
    match table_entry {
        None => Ok(SupportedVarType::String),
        Some(toml::Value::String(value)) if value == "string" => Ok(SupportedVarType::String),
        Some(toml::Value::String(value)) if value == "bool" => Ok(SupportedVarType::Bool),
        Some(toml::Value::String(value)) => Err(ConversionError::InvalidVariableType {
            var_name: var_name.into(),
            value: value.clone(),
        }),
        Some(_) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "type".to_string(),
            correct_type: "String".to_string(),
        }),
    }
}

fn extract_prompt(
    var_name: &str,
    table_entry: Option<&toml::Value>,
) -> Result<String, ConversionError> {
    match table_entry {
        Some(toml::Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "prompt".into(),
            correct_type: "String".into(),
        }),
        None => Err(ConversionError::MissingPrompt {
            var_name: var_name.into(),
        }),
    }
}

fn extract_default(
    var_name: &str,
    var_type: SupportedVarType,
    regex: Option<&Regex>,
    table_entry: Option<&toml::Value>,
    choices: Option<&Vec<String>>,
) -> Result<Option<SupportedVarValue>, ConversionError> {
    match (table_entry, choices, var_type) {
        // no default set
        (None, _, _) => Ok(None),
        // default set without choices
        (Some(toml::Value::Boolean(value)), _, SupportedVarType::Bool) => {
            Ok(Some(SupportedVarValue::Bool(*value)))
        }
        (Some(toml::Value::String(value)), None, SupportedVarType::String) => {
            if let Some(reg) = regex {
                if !reg.is_match(value) {
                    return Err(ConversionError::RegexDoesntMatchField {
                        var_name: var_name.into(),
                        field: "default".to_string(),
                    });
                }
            }
            Ok(Some(SupportedVarValue::String(value.clone())))
        }

        // default and choices set
        // No need to check bool because it always has a choices vec with two values
        (Some(toml::Value::String(value)), Some(choices), SupportedVarType::String) => {
            if !choices.contains(value) {
                Err(ConversionError::InvalidDefault {
                    var_name: var_name.into(),
                    default: value.clone(),
                    choices: choices.clone(),
                })
            } else {
                if let Some(reg) = regex {
                    if !reg.is_match(value) {
                        return Err(ConversionError::RegexDoesntMatchField {
                            var_name: var_name.into(),
                            field: "default".to_string(),
                        });
                    }
                }
                Ok(Some(SupportedVarValue::String(value.clone())))
            }
        }

        // Wrong type of variables
        (Some(_), _, type_name) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "default".to_string(),
            correct_type: match type_name {
                SupportedVarType::Bool => "bool".to_string(),
                SupportedVarType::String => "string".to_string(),
            },
        }),
    }
}

fn extract_choices(
    var_name: &str,
    var_type: SupportedVarType,
    regex: Option<&Regex>,
    table_entry: Option<&toml::Value>,
) -> Result<Option<Vec<String>>, ConversionError> {
    match (table_entry, var_type) {
        (None, SupportedVarType::Bool) => Ok(None),
        (Some(_), SupportedVarType::Bool) => Err(ConversionError::ChoicesOnBool {
            var_name: var_name.into(),
        }),
        (Some(toml::Value::Array(arr)), SupportedVarType::String) if arr.is_empty() => {
            Err(ConversionError::EmptyChoices {
                var_name: var_name.into(),
            })
        }
        (Some(toml::Value::Array(arr)), SupportedVarType::String) => {
            // Checks if very entry in the array is a String
            let converted = arr
                .iter()
                .map(|entry| match entry {
                    toml::Value::String(s) => Ok(s.clone()),
                    _ => Err(()),
                })
                .collect::<Vec<_>>();
            if converted.iter().any(|v| v.is_err()) {
                return Err(ConversionError::WrongTypeParameter {
                    var_name: var_name.into(),
                    parameter: "choices".to_string(),
                    correct_type: "String Array".to_string(),
                });
            }

            let strings = converted
                .iter()
                .cloned()
                .map(|v| v.unwrap())
                .collect::<Vec<_>>();
            // check if regex matches every choice
            if let Some(reg) = regex {
                if strings.iter().any(|v| !reg.is_match(v)) {
                    return Err(ConversionError::RegexDoesntMatchField {
                        var_name: var_name.into(),
                        field: "choices".to_string(),
                    });
                }
            }

            Ok(Some(strings))
        }
        (Some(_), SupportedVarType::String) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "choices".to_string(),
            correct_type: "String Array".to_string(),
        }),
        (None, SupportedVarType::String) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_choices_boolean() {
        let result = extract_choices("foo", SupportedVarType::Bool, None, None);

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn boolean_cant_have_choices() {
        let result = extract_choices(
            "foo",
            SupportedVarType::Bool,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::Boolean(true),
                toml::Value::Boolean(false),
            ])),
        );

        assert_eq!(
            result,
            Err(ConversionError::ChoicesOnBool {
                var_name: "foo".into()
            })
        );
    }

    #[test]
    fn choices_cant_be_an_empty_array() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Array(Vec::new())),
        );

        assert_eq!(
            result,
            Err(ConversionError::EmptyChoices {
                var_name: "foo".into()
            })
        );
    }

    #[test]
    fn choices_array_cant_have_anything_but_strings() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::String("bar".into()),
                toml::Value::Boolean(false),
            ])),
        );

        assert_eq!(
            result,
            Err(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "choices".into(),
                correct_type: "String Array".into()
            })
        );
    }

    #[test]
    fn choices_is_array_string_no_regex_is_fine() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::String("bar".into()),
                toml::Value::String("zoo".into()),
            ])),
        );

        assert_eq!(result, Ok(Some(vec!["bar".to_string(), "zoo".to_string()])));
    }

    #[test]
    fn choices_is_array_string_that_doesnt_match_regex_is_error() {
        let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$").unwrap();

        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            Some(&valid_ident),
            Some(&toml::Value::Array(vec![
                toml::Value::String("0bar".into()),
                toml::Value::String("zoo".into()),
            ])),
        );

        assert_eq!(
            result,
            Err(ConversionError::RegexDoesntMatchField {
                var_name: "foo".into(),
                field: "choices".into()
            })
        );
    }

    #[test]
    fn choices_is_array_string_that_all_match_regex_is_good() {
        let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$").unwrap();

        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            Some(&valid_ident),
            Some(&toml::Value::Array(vec![
                toml::Value::String("bar0".into()),
                toml::Value::String("zoo".into()),
            ])),
        );

        assert_eq!(
            result,
            Ok(Some(vec!["bar0".to_string(), "zoo".to_string()]))
        );
    }

    #[test]
    fn choices_is_not_array_string_is_error() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::String("bar".into())),
        );

        assert_eq!(
            result,
            Err(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "choices".into(),
                correct_type: "String Array".into()
            })
        );
    }

    #[test]
    fn no_choices_for_type_string() {
        let result = extract_choices("foo", SupportedVarType::String, None, None);

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn empty_default_is_fine() {
        let result = extract_default("foo", SupportedVarType::String, None, None, None);

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn default_for_boolean_is_fine() {
        let result = extract_default(
            "foo",
            SupportedVarType::Bool,
            None,
            Some(&toml::Value::Boolean(true)),
            None,
        );

        assert_eq!(result, Ok(Some(SupportedVarValue::Bool(true))))
    }

    #[test]
    fn default_for_string_with_no_choices_and_no_regex() {
        let result = extract_default(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::String("bar".to_string())),
            None,
        );

        assert_eq!(
            result,
            Ok(Some(SupportedVarValue::String("bar".to_string())))
        )
    }

    #[test]
    fn default_for_string_with_no_choices_and_matching_regex() {
        let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$").unwrap();

        let result = extract_default(
            "foo",
            SupportedVarType::String,
            Some(&valid_ident),
            Some(&toml::Value::String("bar".to_string())),
            None,
        );

        assert_eq!(
            result,
            Ok(Some(SupportedVarValue::String("bar".to_string())))
        )
    }

    #[test]
    fn default_for_string_with_no_choices_and_regex_doesnt_match() {
        let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$").unwrap();

        let result = extract_default(
            "foo",
            SupportedVarType::String,
            Some(&valid_ident),
            Some(&toml::Value::String("0bar".to_string())),
            None,
        );

        assert_eq!(
            result,
            Err(ConversionError::RegexDoesntMatchField {
                var_name: "foo".into(),
                field: "default".into()
            })
        )
    }

    #[test]
    fn default_for_string_isnt_on_choices() {
        let result = extract_default(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::String("bar".to_string())),
            Some(&vec!["zoo".to_string(), "far".to_string()]),
        );

        assert_eq!(
            result,
            Err(ConversionError::InvalidDefault {
                var_name: "foo".into(),
                default: "bar".into(),
                choices: vec!["zoo".to_string(), "far".to_string()]
            })
        )
    }

    #[test]
    fn default_for_string_is_on_choices() {
        let result = extract_default(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::String("bar".to_string())),
            Some(&vec!["zoo".to_string(), "bar".to_string()]),
        );

        assert_eq!(result, Ok(Some(SupportedVarValue::String("bar".into()))))
    }

    #[test]
    fn default_for_string_is_on_choices_and_matches_regex() {
        let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$").unwrap();

        let result = extract_default(
            "foo",
            SupportedVarType::String,
            Some(&valid_ident),
            Some(&toml::Value::String("bar".to_string())),
            Some(&vec!["zoo".to_string(), "bar".to_string()]),
        );

        assert_eq!(result, Ok(Some(SupportedVarValue::String("bar".into()))))
    }

    #[test]
    fn default_for_string_only_accepts_strings() {
        let result = extract_default(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Integer(0)),
            None,
        );

        assert_eq!(
            result,
            Err(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "default".into(),
                correct_type: "string".into()
            })
        )
    }

    #[test]
    fn default_for_bool_only_accepts_bool() {
        let result = extract_default(
            "foo",
            SupportedVarType::Bool,
            None,
            Some(&toml::Value::Integer(0)),
            None,
        );

        assert_eq!(
            result,
            Err(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "default".into(),
                correct_type: "bool".into()
            })
        )
    }

    #[test]
    fn prompt_cant_be_empty() {
        let result = extract_prompt("foo", None);

        assert_eq!(
            result,
            Err(ConversionError::MissingPrompt {
                var_name: "foo".into(),
            })
        )
    }

    #[test]
    fn prompt_must_be_string() {
        let result = extract_prompt("foo", Some(&toml::Value::Integer(0)));

        assert_eq!(
            result,
            Err(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "prompt".into(),
                correct_type: "String".into()
            })
        )
    }

    #[test]
    fn prompt_as_string_is_ok() {
        let result = extract_prompt("foo", Some(&toml::Value::String("hello world".into())));

        assert_eq!(result, Ok("hello world".into()))
    }

    #[test]
    fn empty_type_is_string() {
        let result = extract_type("foo", None);

        assert_eq!(result, Ok(SupportedVarType::String));
    }

    #[test]
    fn type_must_be_string_type() {
        let result = extract_type("foo", Some(&toml::Value::Integer(0)));

        assert_eq!(
            result,
            Err(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "type".into(),
                correct_type: "String".into()
            })
        );
    }

    #[test]
    fn type_must_either_be_string_or_bool() {
        let result_bool = extract_type("foo", Some(&toml::Value::String("bool".into())));
        let result_string = extract_type("foo", Some(&toml::Value::String("string".into())));
        let result_err = extract_type("foo", Some(&toml::Value::String("bar".into())));

        assert_eq!(result_bool, Ok(SupportedVarType::Bool));
        assert_eq!(result_string, Ok(SupportedVarType::String));
        assert_eq!(
            result_err,
            Err(ConversionError::InvalidVariableType {
                var_name: "foo".into(),
                value: "bar".into()
            })
        )
    }

    #[test]
    fn bools_cant_have_regex() {
        let result = extract_regex(
            "foo",
            SupportedVarType::Bool,
            Some(&toml::Value::String("".into())),
        );

        assert!(result.is_err())
    }

    #[test]
    fn no_regex_is_ok() {
        let result_bool = extract_regex("foo", SupportedVarType::Bool, None);
        let result_string = extract_regex("foo", SupportedVarType::String, None);

        assert!(result_bool.is_ok());
        assert!(result_string.is_ok())
    }

    #[test]
    fn strings_can_have_regex() {
        let result = extract_regex(
            "foo",
            SupportedVarType::String,
            Some(&toml::Value::String("^([a-zA-Z][a-zA-Z0-9_-]+)$".into())),
        );

        assert!(result.is_ok())
    }

    #[test]
    fn invalid_regex_is_err() {
        let result = extract_regex(
            "foo",
            SupportedVarType::String,
            Some(&toml::Value::String("*".into())),
        );

        assert!(result.is_err())
    }

    #[test]
    fn block_invalid_key_names() {
        let result =
            try_key_value_into_slot("project-name", &toml::Value::Table(Default::default()));

        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(
            result,
            ConversionError::InvalidPlaceholderName {
                var_name: "project-name".into()
            }
        );

        let result = try_key_value_into_slot("crate_name", &toml::Value::Table(Default::default()));

        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(
            result,
            ConversionError::InvalidPlaceholderName {
                var_name: "crate_name".into()
            }
        );
    }

    #[test]
    fn only_tables_as_placeholder_values() {
        let result = try_key_value_into_slot("foo", &toml::Value::Integer(Default::default()));

        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(
            result,
            ConversionError::InvalidPlaceholderFormat {
                var_name: "foo".into()
            }
        );
    }
}
