use anyhow::Result;
use indexmap::IndexMap;
use liquid_core::model::map::Entry;
use liquid_core::{Value, ValueView};
use log::info;
use regex::Regex;
use thiserror::Error;

use crate::emoji;
use console::style;

use crate::{
    config::{Config, TemplateSlotsTable},
    interactive::LIST_SEP,
    template::LiquidObjectResource,
};

#[derive(Debug)]
pub struct TemplateSlots {
    pub(crate) var_name: String,
    pub(crate) var_info: VarInfo,
    pub(crate) prompt: Prompt,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub(crate) _raw: String,
    pub(crate) styled: String,
    pub(crate) styled_with_default: String,
    pub(crate) with_default: String,
}
impl Prompt {
    pub(crate) fn new(prompt: impl Into<String>, default: Option<String>) -> Self {
        let prompt = prompt.into();
        let styled = format!("{} {}", emoji::SHRUG, style(&prompt).bold(),);
        let styled_with_default = format!(
            "{styled}{}",
            default
                .as_ref()
                .map(|default| format!(" [default: {}]", style(default).bold()))
                .unwrap_or_default()
        );
        let with_default = format!(
            "{prompt}{}",
            default
                .as_ref()
                .map(|default| format!(" [default: {default}]"))
                .unwrap_or_default()
        );
        Self {
            _raw: prompt,
            styled,
            styled_with_default,
            with_default,
        }
    }
}

impl From<&str> for Prompt {
    fn from(value: &str) -> Self {
        Self::new(value, None)
    }
}

impl From<String> for Prompt {
    fn from(value: String) -> Self {
        Self::new(&value, None)
    }
}

impl From<&String> for Prompt {
    fn from(value: &String) -> Self {
        Self::new(value, None)
    }
}

/// Information needed to prompt for a typed value
/// Editor will never have choices
#[derive(Debug, Clone)]
pub enum VarInfo {
    Array { entry: Box<ArrayEntry> },
    Bool { default: Option<bool> },
    String { entry: Box<StringEntry> },
}

#[derive(Debug, Clone)]
pub struct ArrayEntry {
    pub(crate) default: Option<Vec<String>>,
    pub(crate) choices: Vec<Choice>,
}

#[derive(Debug, Clone)]
pub struct StringEntry {
    pub(crate) default: Option<String>,
    pub(crate) kind: StringKind,
    pub(crate) regex: Option<Regex>,
}

#[derive(Debug, Clone)]
pub enum StringKind {
    Choices(Vec<Choice>),
    String,
    Editor,
    Text,
}

/// A single selectable entry of a `choices` placeholder.
///
/// A choice can be written either as a plain string, in which case the label
/// shown to the user and the value handed to the template are identical, or as
/// a table carrying an explicit `value` and an optional display `label`:
///
/// ```toml
/// choices = [
///     { value = "recommended", label = "1.3.7 (recommended)" },
///     "experimental",
/// ]
/// ```
///
/// The `label` is only ever used for display; matching against `default` and
/// the value substituted into the template always use `value`.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub(crate) value: String,
    pub(crate) label: String,
}

impl Choice {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            label: value.clone(),
            value,
        }
    }

    fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
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
    #[error("choice entry of `{var_name}` must be a string or a table with a string `value` field and an optional string `label`")]
    InvalidChoiceEntry { var_name: String },
    #[error("default is `{default}`, but is not a valid value in choices array `{choices:?}` for `{var_name}`")]
    InvalidDefault {
        var_name: String,
        default: String,
        choices: Vec<String>,
    },
    #[error(
        "invalid type for variable `{var_name}`: `{value}` possible values are `bool`, `string`, `text` and `editor`"
    )]
    InvalidVariableType { var_name: String, value: String },
    #[error("{var_type} type does not support `choices` field")]
    UnsupportedChoices { var_type: String },
    #[error("bool type does not support `regex` field")]
    RegexOnBool { var_name: String },
    #[error(
        "variable `{var_name}` is missing default value in config file running in silent mode"
    )]
    MissingDefaultValueForPlaceholderVariable { var_name: String },
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
    Array(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SupportedVarType {
    Bool,
    String,
    Editor,
    Text,
    Array,
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

pub fn show_project_variables_with_value(template_object: &LiquidObjectResource, config: &Config) {
    let template_slots = config
        .placeholders
        .as_ref()
        .map(try_into_template_slots)
        .unwrap_or_else(|| Ok(IndexMap::new()))
        .unwrap_or_default();

    template_slots
        .iter()
        .filter(|(k, _)| template_object.lock().unwrap().borrow().contains_key(**k))
        .for_each(|(k, v)| {
            let name = v.var_name.as_str();
            let value = template_object
                .lock()
                .unwrap()
                .borrow()
                .get(*k)
                .unwrap()
                .to_kstr()
                .to_string();
            info!(
                "{} {} (placeholder provided by cli argument)",
                emoji::WRENCH,
                style(format!("{name}: {value:?}")).bold(),
            )
        });
}

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

    let var_info = match var_type {
        SupportedVarType::Bool => VarInfo::Bool {
            default: if let Some(SupportedVarValue::Bool(value)) = default_choice {
                Some(value)
            } else {
                None
            },
        },
        SupportedVarType::String => VarInfo::String {
            entry: Box::new(StringEntry {
                default: if let Some(SupportedVarValue::String(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                kind: choices.map_or(StringKind::String, StringKind::Choices),
                regex,
            }),
        },
        SupportedVarType::Editor => VarInfo::String {
            entry: Box::new(StringEntry {
                default: if let Some(SupportedVarValue::String(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                kind: StringKind::Editor,
                regex,
            }),
        },
        SupportedVarType::Array => VarInfo::Array {
            entry: Box::new(ArrayEntry {
                default: if let Some(SupportedVarValue::Array(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                choices: choices.unwrap_or_default(),
            }),
        },
        SupportedVarType::Text => VarInfo::String {
            entry: Box::new(StringEntry {
                default: if let Some(SupportedVarValue::String(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                kind: StringKind::Text,
                regex,
            }),
        },
    };
    Ok(TemplateSlots {
        var_name: key.to_string(),
        var_info,
        prompt: prompt.into(),
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
        (
            SupportedVarType::String | SupportedVarType::Editor | SupportedVarType::Text,
            Some(toml::Value::String(value)),
        ) => match Regex::new(value) {
            Ok(regex) => Ok(Some(regex)),
            Err(e) => Err(ConversionError::InvalidRegex {
                var_name: var_name.into(),
                regex: value.clone(),
                error: e,
            }),
        },
        (
            SupportedVarType::String
            | SupportedVarType::Editor
            | SupportedVarType::Text
            | SupportedVarType::Array,
            Some(_),
        ) => Err(ConversionError::WrongTypeParameter {
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
        Some(toml::Value::String(value)) if value == "editor" => Ok(SupportedVarType::Editor),
        Some(toml::Value::String(value)) if value == "text" => Ok(SupportedVarType::Text),
        Some(toml::Value::String(value)) if value == "bool" => Ok(SupportedVarType::Bool),
        Some(toml::Value::String(value)) if value == "array" => Ok(SupportedVarType::Array),
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
    choices: Option<&Vec<Choice>>,
) -> Result<Option<SupportedVarValue>, ConversionError> {
    match (table_entry, choices, var_type) {
        // no default set
        (None, _, _) => Ok(None),
        // default set without choices
        (Some(toml::Value::Boolean(value)), _, SupportedVarType::Bool) => {
            Ok(Some(SupportedVarValue::Bool(*value)))
        }
        (
            Some(toml::Value::String(value)),
            None,
            SupportedVarType::String | SupportedVarType::Editor | SupportedVarType::Text,
        ) => {
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
        (
            Some(toml::Value::String(value)),
            Some(choices),
            SupportedVarType::String | SupportedVarType::Editor | SupportedVarType::Text,
        ) => {
            if !choices.iter().any(|c| &c.value == value) {
                Err(ConversionError::InvalidDefault {
                    var_name: var_name.into(),
                    default: value.clone(),
                    choices: choice_values(choices),
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
        (Some(toml::Value::Array(defaults)), Some(choices), SupportedVarType::Array) => {
            let default_string_array: Vec<String> = defaults
                .iter()
                .filter(|f| !(f.is_table() && f.is_array()))
                .map(|f| f.as_str().unwrap_or_default().to_string())
                .collect();
            if default_string_array
                .iter()
                .all(|v| choices.iter().any(|c| &c.value == v))
            {
                Ok(Some(SupportedVarValue::Array(default_string_array.clone())))
            } else {
                Err(ConversionError::InvalidDefault {
                    var_name: var_name.into(),
                    default: default_string_array.join(LIST_SEP),
                    choices: choice_values(choices),
                })
            }
        }

        // Wrong type of variables
        (Some(_), _, type_name) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "default".to_string(),
            correct_type: match type_name {
                SupportedVarType::Bool => "bool".to_string(),
                SupportedVarType::String => "string".to_string(),
                SupportedVarType::Editor => "editor".to_string(),
                SupportedVarType::Text => "text".to_string(),
                SupportedVarType::Array => "array".to_string(),
            },
        }),
    }
}

/// Collect the raw values of a set of choices, discarding the display labels.
fn choice_values(choices: &[Choice]) -> Vec<String> {
    choices.iter().map(|c| c.value.clone()).collect()
}

/// Turn a single `choices` array entry into a [`Choice`].
///
/// Accepts either a plain string (value == label) or a table with a string
/// `value` and an optional string `label`.
fn convert_choice_entry(var_name: &str, entry: &toml::Value) -> Result<Choice, ConversionError> {
    match entry {
        toml::Value::String(s) => Ok(Choice::new(s.clone())),
        toml::Value::Table(table) => {
            let value = match table.get("value") {
                Some(toml::Value::String(value)) => value.clone(),
                _ => {
                    return Err(ConversionError::InvalidChoiceEntry {
                        var_name: var_name.into(),
                    })
                }
            };
            match table.get("label") {
                Some(toml::Value::String(label)) => Ok(Choice::with_label(value, label.clone())),
                None => Ok(Choice::new(value)),
                Some(_) => Err(ConversionError::InvalidChoiceEntry {
                    var_name: var_name.into(),
                }),
            }
        }
        _ => Err(ConversionError::InvalidChoiceEntry {
            var_name: var_name.into(),
        }),
    }
}

fn extract_choices(
    var_name: &str,
    var_type: SupportedVarType,
    regex: Option<&Regex>,
    table_entry: Option<&toml::Value>,
) -> Result<Option<Vec<Choice>>, ConversionError> {
    match (table_entry, var_type) {
        (
            None,
            SupportedVarType::Bool
            | SupportedVarType::Editor
            | SupportedVarType::Text
            | SupportedVarType::Array
            | SupportedVarType::String,
        ) => Ok(None),
        (Some(_), SupportedVarType::Bool | SupportedVarType::Editor | SupportedVarType::Text) => {
            Err(ConversionError::UnsupportedChoices {
                var_type: format!("{var_type:?}"),
            })
        }
        (Some(toml::Value::Array(arr)), SupportedVarType::String) if arr.is_empty() => {
            Err(ConversionError::EmptyChoices {
                var_name: var_name.into(),
            })
        }
        (Some(toml::Value::Array(arr)), SupportedVarType::String | SupportedVarType::Array) => {
            let choices = arr
                .iter()
                .map(|entry| convert_choice_entry(var_name, entry))
                .collect::<Result<Vec<Choice>, _>>()?;

            // check that the regex matches every choice value (string type only)
            if var_type == SupportedVarType::String {
                if let Some(reg) = regex {
                    if choices.iter().any(|c| !reg.is_match(&c.value)) {
                        return Err(ConversionError::RegexDoesntMatchField {
                            var_name: var_name.into(),
                            field: "choices".to_string(),
                        });
                    }
                }
            }

            Ok(Some(choices))
        }
        (Some(_), SupportedVarType::String | SupportedVarType::Array) => {
            Err(ConversionError::WrongTypeParameter {
                var_name: var_name.into(),
                parameter: "choices".to_string(),
                correct_type: "String Array".to_string(),
            })
        }
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
    fn no_choices_editor() {
        let result = extract_choices("foo", SupportedVarType::Editor, None, None);

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn no_choices_text() {
        let result = extract_choices("foo", SupportedVarType::Text, None, None);

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
            Err(ConversionError::UnsupportedChoices {
                var_type: "Bool".into()
            })
        );
    }

    #[test]
    fn editor_cant_have_regex() {
        let result = extract_regex(
            "foo",
            SupportedVarType::Editor,
            Some(&toml::Value::Array(vec![
                toml::Value::Boolean(true),
                toml::Value::Boolean(false),
            ])),
        );

        assert_eq!(
            result.err(),
            Some(ConversionError::WrongTypeParameter {
                var_name: "foo".to_string(),
                parameter: "regex".to_string(),
                correct_type: "String".to_string()
            })
        );
    }

    #[test]
    fn cant_have_default_wrong_type() {
        let result = extract_default(
            "foo",
            SupportedVarType::Array,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::Boolean(true),
                toml::Value::Boolean(false),
            ])),
            None,
        );

        assert_eq!(
            result.err(),
            Some(ConversionError::WrongTypeParameter {
                var_name: "foo".into(),
                parameter: "default".into(),
                correct_type: "array".into()
            })
        );
    }

    #[test]
    fn text_cant_have_choices() {
        let result = extract_choices(
            "foo",
            SupportedVarType::Text,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::Boolean(true),
                toml::Value::Boolean(false),
            ])),
        );

        assert_eq!(
            result,
            Err(ConversionError::UnsupportedChoices {
                var_type: "Text".into()
            })
        );
    }

    #[test]
    fn editor_cant_have_choices() {
        let result = extract_choices(
            "foo",
            SupportedVarType::Editor,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::Boolean(true),
                toml::Value::Boolean(false),
            ])),
        );
        assert_eq!(
            result,
            Err(ConversionError::UnsupportedChoices {
                var_type: "Editor".into()
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
    fn multi_choices_can_be_an_empty_array() {
        let result = extract_choices(
            "foo",
            SupportedVarType::Array,
            None,
            Some(&toml::Value::Array(Vec::new())),
        );

        assert_eq!(result, Ok(Some(Vec::new())));
    }

    #[test]
    fn choices_array_cant_have_anything_but_strings_or_tables() {
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
            Err(ConversionError::InvalidChoiceEntry {
                var_name: "foo".into(),
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

        assert_eq!(
            result,
            Ok(Some(vec![Choice::new("bar"), Choice::new("zoo")]))
        );
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
            Ok(Some(vec![Choice::new("bar0"), Choice::new("zoo")]))
        );
    }

    fn choice_table(value: Option<&str>, label: Option<&str>) -> toml::Value {
        let mut map = toml::map::Map::new();
        if let Some(value) = value {
            map.insert("value".into(), toml::Value::String(value.into()));
        }
        if let Some(label) = label {
            map.insert("label".into(), toml::Value::String(label.into()));
        }
        toml::Value::Table(map)
    }

    #[test]
    fn choices_accept_value_label_tables_mixed_with_plain_strings() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Array(vec![
                choice_table(Some("recommended"), Some("1.3.7 (recommended)")),
                toml::Value::String("experimental".into()),
            ])),
        );

        assert_eq!(
            result,
            Ok(Some(vec![
                Choice::with_label("recommended", "1.3.7 (recommended)"),
                Choice::new("experimental"),
            ]))
        );
    }

    #[test]
    fn choice_table_without_label_defaults_label_to_value() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Array(vec![choice_table(Some("bar"), None)])),
        );

        assert_eq!(result, Ok(Some(vec![Choice::new("bar")])));
    }

    #[test]
    fn choice_table_without_value_is_error() {
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::Array(vec![choice_table(
                None,
                Some("just a label"),
            )])),
        );

        assert_eq!(
            result,
            Err(ConversionError::InvalidChoiceEntry {
                var_name: "foo".into(),
            })
        );
    }

    #[test]
    fn choices_regex_is_checked_against_value_not_label() {
        let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$").unwrap();

        // The value is a valid identifier while the label is not; only the
        // value has to satisfy the regex.
        let result = extract_choices(
            "foo",
            SupportedVarType::String,
            Some(&valid_ident),
            Some(&toml::Value::Array(vec![choice_table(
                Some("recommended"),
                Some("1.3.7 (recommended)"),
            )])),
        );

        assert_eq!(
            result,
            Ok(Some(vec![Choice::with_label(
                "recommended",
                "1.3.7 (recommended)"
            )]))
        );
    }

    #[test]
    fn multi_choices_accept_value_label_tables() {
        let result = extract_choices(
            "foo",
            SupportedVarType::Array,
            None,
            Some(&toml::Value::Array(vec![
                choice_table(Some("serde"), Some("Serde (serialization)")),
                toml::Value::String("logging".into()),
            ])),
        );

        assert_eq!(
            result,
            Ok(Some(vec![
                Choice::with_label("serde", "Serde (serialization)"),
                Choice::new("logging"),
            ]))
        );
    }

    #[test]
    fn default_is_matched_against_choice_value_not_label() {
        let choices = vec![Choice::with_label("recommended", "1.3.7 (recommended)")];

        let result = extract_default(
            "foo",
            SupportedVarType::String,
            None,
            Some(&toml::Value::String("recommended".to_string())),
            Some(&choices),
        );

        assert_eq!(
            result,
            Ok(Some(SupportedVarValue::String("recommended".into())))
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
    fn multi_choices_table_without_value_is_error() {
        let result = extract_choices(
            "foo",
            SupportedVarType::Array,
            None,
            Some(&toml::Value::Array(vec![
                toml::Value::String("bar0".into()),
                toml::Value::Table(toml::map::Map::new()),
            ])),
        );

        assert_eq!(
            result,
            Err(ConversionError::InvalidChoiceEntry {
                var_name: "foo".into(),
            })
        );
    }

    #[test]
    fn multi_choices_wrong_default_type() {
        let result = extract_default(
            "foo",
            SupportedVarType::Array,
            None,
            Some(&toml::Value::Array(vec![toml::Value::String(
                "true".into(),
            )])),
            Some(&vec![Choice::new("bar0")]),
        );

        assert_eq!(
            result,
            Err(ConversionError::InvalidDefault {
                var_name: "foo".into(),
                default: "true".to_string(),
                choices: vec!["bar0".to_string()]
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
            Some(&vec![Choice::new("zoo"), Choice::new("far")]),
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
            Some(&vec![Choice::new("zoo"), Choice::new("bar")]),
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
            Some(&vec![Choice::new("zoo"), Choice::new("bar")]),
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
