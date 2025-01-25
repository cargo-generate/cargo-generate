use crate::{
    emoji,
    project_variables::{ArrayEntry, Prompt, StringEntry, StringKind, TemplateSlots, VarInfo},
};
use anyhow::{anyhow, bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use dialoguer::{Editor, Input};
use liquid_core::Value;
use log::warn;
use std::{
    borrow::Cow,
    io::{stdin, Read},
    ops::Index,
    str::FromStr,
};

const LIST_SEP: &str = ",";

pub fn name() -> Result<String> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let project_var = TemplateSlots {
        var_name: "crate_name".into(),
        prompt: "Project Name".into(),
        var_info: VarInfo::String {
            entry: Box::new(StringEntry {
                default: None,
                kind: StringKind::String,
                regex: Some(valid_ident),
            }),
        },
    };
    prompt_and_check_variable(&project_var, None)
}

pub fn user_question(
    prompt: &Prompt,
    default: &Option<String>,
    kind: &StringKind,
) -> Result<String> {
    match kind {
        StringKind::String => {
            let mut i = Input::<String>::new().with_prompt(&prompt.styled_with_default);
            if let Some(s) = default {
                i = i.default(s.to_owned());
            }
            i.interact().map_err(Into::<anyhow::Error>::into)
        }
        StringKind::Editor => {
            println!("{} (in Editor)", prompt.styled_with_default);
            Editor::new()
                .edit(&prompt.with_default)?
                .or_else(|| default.clone())
                .ok_or(anyhow!("Aborted Editor without saving !"))
        }
        StringKind::Text => {
            println!(
                "{} (press Ctrl+d to stop reading)",
                prompt.styled_with_default
            );
            let mut buffer = String::new();
            stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        StringKind::Choices(_) => {
            unreachable!("StringKind::Choices should be handled in the parent")
        }
    }
}

pub fn prompt_and_check_variable(
    variable: &TemplateSlots,
    provided_value: Option<String>,
) -> Result<String> {
    match &variable.var_info {
        VarInfo::Bool { default } => handle_bool_input(provided_value, &variable.prompt, default),
        VarInfo::String { entry } => match &entry.kind {
            StringKind::Choices(choices) => handle_choice_input(
                provided_value,
                &variable.var_name,
                choices,
                entry,
                &variable.prompt,
            ),
            StringKind::String | StringKind::Text | StringKind::Editor => {
                handle_string_input(provided_value, &variable.var_name, entry, &variable.prompt)
            }
        },
        VarInfo::Array { entry } => {
            handle_multi_select_input(provided_value, &variable.var_name, entry, &variable.prompt)
        }
    }
}

pub fn variable(variable: &TemplateSlots, provided_value: Option<&impl ToString>) -> Result<Value> {
    let user_entry = prompt_and_check_variable(variable, provided_value.map(|v| v.to_string()))?;
    match &variable.var_info {
        VarInfo::Bool { .. } => {
            let as_bool = user_entry.parse::<bool>()?;
            Ok(Value::Scalar(as_bool.into()))
        }
        VarInfo::String { .. } => Ok(Value::Scalar(user_entry.into())),
        VarInfo::Array { .. } => Ok(Value::Array(
            user_entry
                .split(LIST_SEP)
                .map(|s| Value::Scalar(s.to_string().into()))
                .collect(),
        )),
    }
}

fn handle_string_input(
    provided_value: Option<String>,
    var_name: &str,
    entry: &StringEntry,
    prompt: &Prompt,
) -> Result<String> {
    if let Some(value) = provided_value {
        if entry
            .regex
            .as_ref()
            .map(|ex| ex.is_match(&value))
            .unwrap_or(true)
        {
            return Ok(value);
        }
        bail!(
            "{} {} \"{}\" {}",
            emoji::WARN,
            style("Sorry,").bold().red(),
            style(&value).bold().yellow(),
            style(format!("is not a valid value for {var_name}"))
                .bold()
                .red()
        )
    };
    let mut prompt: Cow<'_, Prompt> = Cow::Borrowed(prompt);
    match &entry.regex {
        Some(regex) => loop {
            let user_entry = user_question(&prompt, &entry.default, &entry.kind)?;
            if regex.is_match(&user_entry) {
                break Ok(user_entry);
            }
            // the user won't see the error in stdout if in a editor
            match entry.kind {
                StringKind::Editor => {
                    // Editor use with_default
                    prompt.to_mut().with_default = format!(
                        "{}: \"{user_entry}\" is not a valid value for `{var_name}`",
                        prompt
                            .with_default
                            .split_once(':')
                            .map(|t| t.0)
                            .unwrap_or(&prompt.with_default)
                    );
                }
                _ => {
                    warn!(
                        "{} \"{}\" {}",
                        style("Sorry,").bold().red(),
                        style(&user_entry).bold().yellow(),
                        style(format!("is not a valid value for {var_name}"))
                            .bold()
                            .red()
                    );
                }
            };
        },
        None => Ok(user_question(&prompt, &entry.default, &entry.kind)?),
    }
}

fn handle_choice_input(
    provided_value: Option<String>,
    var_name: &str,
    choices: &Vec<String>,
    entry: &StringEntry,
    prompt: &Prompt,
) -> Result<String> {
    match provided_value {
        Some(value) => {
            if choices.contains(&value) {
                Ok(value)
            } else {
                bail!(
                    "{} {} \"{}\" {}",
                    emoji::WARN,
                    style("Sorry,").bold().red(),
                    style(&value).bold().yellow(),
                    style(format!("is not a valid value for {var_name}"))
                        .bold()
                        .red(),
                )
            }
        }
        None => {
            let default = entry
                .default
                .as_ref()
                .map_or(0, |default| choices.binary_search(default).unwrap_or(0));
            let chosen = Select::with_theme(&ColorfulTheme::default())
                .items(choices)
                .with_prompt(&prompt.styled)
                .default(default)
                .interact()?;

            Ok(choices.index(chosen).to_string())
        }
    }
}

// simple function so we can easily get more complicated later if we need to
fn parse_list(provided_value: &str) -> Vec<String> {
    provided_value.split(',').map(|s| s.to_string()).collect()
}

fn check_provided_selections(provided_value: &str, choices: &[String]) -> Result<String, String> {
    let list = parse_list(provided_value);
    let (ok_entries, bad_entries): (Vec<String>, Vec<String>) =
        list.iter().cloned().partition(|e| choices.contains(e));
    if bad_entries.is_empty() {
        Ok(ok_entries.join(LIST_SEP))
    } else {
        Err(bad_entries.join(LIST_SEP))
    }
}

fn handle_multi_select_input(
    provided_value: Option<String>,
    var_name: &str,
    entry: &ArrayEntry,
    prompt: &Prompt,
) -> Result<String> {
    let val = match provided_value {
        // value is just povided
        Some(value) => value,
        // no value is provided so we have to be smarter
        None => {
            let mut selected_by_default = Vec::<bool>::with_capacity(entry.choices.len());
            match &entry.default {
                // if no defaults are provided everything is disselected by default
                None => {
                    selected_by_default.resize(entry.choices.len(), false);
                }
                Some(default_choices) => {
                    for choice in &entry.choices {
                        selected_by_default.push(default_choices.contains(choice));
                    }
                }
            };

            let choice_indeces = MultiSelect::with_theme(&ColorfulTheme::default())
                .items(&entry.choices)
                .with_prompt(&prompt.styled)
                .defaults(&selected_by_default)
                .interact()?;

            choice_indeces
                .iter()
                .filter_map(|idx| entry.choices.get(*idx))
                .cloned()
                .collect::<Vec<String>>()
                .join(LIST_SEP)
        }
    };

    match check_provided_selections(&val, &entry.choices) {
        Ok(s) => Ok(s),
        Err(s) => bail!(
            "{} {} \"{}\" {}",
            emoji::WARN,
            style("Sorry,").bold().red(),
            style(&s).bold().yellow(),
            style(format!("are not a valid values for {var_name}"))
                .bold()
                .red(),
        ),
    }
}

fn handle_bool_input(
    provided_value: Option<String>,
    prompt: &Prompt,
    default: &Option<bool>,
) -> Result<String> {
    match provided_value {
        Some(value) => {
            let value = bool::from_str(&value.to_lowercase())?;
            Ok(value.to_string())
        }
        None => {
            let choices = [false.to_string(), true.to_string()];
            let chosen = Select::with_theme(&ColorfulTheme::default())
                .items(&choices)
                .with_prompt(&prompt.styled)
                .default(usize::from(default.unwrap_or(false)))
                .interact()?;

            Ok(choices.index(chosen).to_string())
        }
    }
}
