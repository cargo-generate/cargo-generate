use crate::{
    emoji,
    project_variables::{Prompt, StringEntry, StringKind, TemplateSlots, VarInfo},
};
use anyhow::{anyhow, bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use dialoguer::{Editor, Input};
use liquid_core::Value;
use log::warn;
use std::{
    borrow::Cow,
    io::{stdin, Read},
    ops::Index,
    str::FromStr,
};

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
        VarInfo::Array { entry } => todo!(),
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
        VarInfo::Array { entry } => todo!(),
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

fn handle_multi_select_input(
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
