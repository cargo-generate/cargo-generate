use crate::{
    emoji,
    project_variables::{StringEntry, TemplateSlots, VarInfo},
};
use anyhow::{bail, Result};
use console::style;
use dialoguer::Input;
use dialoguer::{theme::ColorfulTheme, Select};
use liquid_core::Value;
use std::{ops::Index, str::FromStr};

pub fn name() -> Result<String> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let project_var = TemplateSlots {
        var_name: "crate_name".into(),
        prompt: "Project Name".into(),
        var_info: VarInfo::String {
            entry: Box::new(StringEntry {
                default: None,
                choices: None,
                regex: Some(valid_ident),
            }),
        },
    };
    prompt_and_check_variable(&project_var, None)
}

pub fn user_question(prompt: &str, default: &Option<String>) -> Result<String> {
    let mut i = Input::<String>::new();
    i.with_prompt(prompt.to_string());
    if let Some(s) = default {
        i.default(s.to_owned());
    }
    i.interact().map_err(Into::<anyhow::Error>::into)
}

pub fn prompt_and_check_variable(
    variable: &TemplateSlots,
    provided_value: Option<String>,
) -> Result<String> {
    let prompt = format!("{} {}", emoji::SHRUG, style(&variable.prompt).bold(),);

    match &variable.var_info {
        VarInfo::Bool { default } => handle_bool_input(provided_value, &prompt, default),
        VarInfo::String { entry } => match &entry.choices {
            Some(choices) => {
                handle_choice_input(provided_value, &variable.var_name, choices, entry, &prompt)
            }
            None => handle_string_input(provided_value, &variable.var_name, entry, &prompt),
        },
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
    }
}

fn handle_string_input(
    provided_value: Option<String>,
    var_name: &str,
    entry: &StringEntry,
    prompt: &str,
) -> Result<String> {
    match provided_value {
        Some(value) => {
            if entry
                .regex
                .as_ref()
                .map(|ex| ex.is_match(&value))
                .unwrap_or(true)
            {
                Ok(value)
            } else {
                bail!(
                    "{} {} \"{}\" {}",
                    emoji::WARN,
                    style("Sorry,").bold().red(),
                    style(&value).bold().yellow(),
                    style(format!("is not a valid value for {}", var_name))
                        .bold()
                        .red()
                )
            }
        }
        None => {
            let prompt = format!(
                "{} {}",
                prompt,
                match &entry.default {
                    Some(d) => format!("[default: {}]", style(d).bold()),
                    None => "".into(),
                }
            );
            let default = entry.default.as_ref().map(|v| v.into());

            match &entry.regex {
                Some(regex) => loop {
                    let user_entry = user_question(prompt.as_str(), &default)?;
                    if regex.is_match(&user_entry) {
                        break Ok(user_entry);
                    }
                    eprintln!(
                        "{} {} \"{}\" {}",
                        emoji::WARN,
                        style("Sorry,").bold().red(),
                        style(&user_entry).bold().yellow(),
                        style(format!("is not a valid value for {}", var_name))
                            .bold()
                            .red()
                    );
                },
                None => Ok(user_question(prompt.as_str(), &default)?),
            }
        }
    }
}

fn handle_choice_input(
    provided_value: Option<String>,
    var_name: &str,
    choices: &Vec<String>,
    entry: &StringEntry,
    prompt: &str,
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
                    style(format!("is not a valid value for {}", var_name))
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
                .with_prompt(prompt)
                .default(default)
                .interact()?;

            Ok(choices.index(chosen).to_string())
        }
    }
}

fn handle_bool_input(
    provided_value: Option<String>,
    prompt: &str,
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
                .with_prompt(prompt)
                .default(if default.unwrap_or(false) { 1 } else { 0 })
                .interact()?;

            Ok(choices.index(chosen).to_string())
        }
    }
}
