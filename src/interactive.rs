use crate::{
    emoji,
    project_variables::{ArrayEntry, Prompt, StringEntry, StringKind, TemplateSlots, VarInfo},
};
use anyhow::{anyhow, bail, Result};
use console::style;
use liquid_core::Value;
use std::{
    borrow::Cow,
    io::{stdin, Read, Write},
    str::FromStr,
};

pub const LIST_SEP: &str = ",";

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
            let mut input = cliclack::input(&prompt.raw);
            if let Some(s) = default {
                input = input.default_input(s).placeholder(s);
            }
            let result: String = input.interact()?;
            Ok(result)
        }
        StringKind::Editor => {
            cliclack::log::info(format!("{} (in Editor)", &prompt.raw))?;
            open_editor(&prompt.with_default)?
                .or_else(|| default.clone())
                .ok_or_else(|| anyhow!("Aborted Editor without saving!"))
        }
        StringKind::Text => {
            cliclack::log::info(format!("{} (press Ctrl+d to stop reading)", &prompt.raw))?;
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
        VarInfo::Array { .. } => {
            let items = if user_entry.is_empty() {
                Vec::new()
            } else {
                user_entry
                    .split(LIST_SEP)
                    .map(|s| Value::Scalar(s.to_string().into()))
                    .collect()
            };

            Ok(Value::Array(items))
        }
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
    }

    match &entry.kind {
        StringKind::String => {
            let mut input = cliclack::input(&prompt.raw);
            if let Some(s) = &entry.default {
                input = input.default_input(s).placeholder(s);
            }
            if let Some(regex) = &entry.regex {
                let regex = regex.clone();
                let vn = var_name.to_string();
                input = input.validate(move |val: &String| {
                    if regex.is_match(val) {
                        Ok(())
                    } else {
                        Err(format!("\"{}\" is not a valid value for {}", val, vn))
                    }
                });
            }
            let result: String = input.interact()?;
            Ok(result)
        }
        kind @ (StringKind::Editor | StringKind::Text) => {
            let mut prompt: Cow<'_, Prompt> = Cow::Borrowed(prompt);
            match &entry.regex {
                Some(regex) => loop {
                    let user_entry = user_question(&prompt, &entry.default, kind)?;
                    if regex.is_match(&user_entry) {
                        break Ok(user_entry);
                    }
                    match kind {
                        StringKind::Editor => {
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
                            cliclack::log::warning(format!(
                                "\"{}\" is not a valid value for {}",
                                &user_entry, var_name
                            ))?;
                        }
                    };
                },
                None => user_question(&prompt, &entry.default, kind),
            }
        }
        StringKind::Choices(_) => {
            unreachable!("StringKind::Choices should be handled in the parent")
        }
    }
}

fn handle_choice_input(
    provided_value: Option<String>,
    var_name: &str,
    choices: &[String],
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
            let default_value = entry
                .default
                .as_ref()
                .and_then(|d| choices.iter().find(|c| *c == d).cloned())
                .unwrap_or_else(|| choices[0].clone());

            let mut select = cliclack::select(&prompt.raw);
            for choice in choices {
                let hint = if *choice == default_value {
                    "default"
                } else {
                    ""
                };
                select = select.item(choice.clone(), choice, hint);
            }
            select = select.initial_value(default_value);

            let chosen: String = select.interact()?;
            Ok(chosen)
        }
    }
}

// simple function so we can easily get more complicated later if we need to
fn parse_list(provided_value: &str) -> Vec<String> {
    provided_value
        .split(LIST_SEP)
        .filter(|e| !e.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn check_provided_selections(
    provided_value: &str,
    choices: &[String],
) -> Result<Vec<String>, Vec<String>> {
    let list = parse_list(provided_value);
    if list.is_empty() {
        return Ok(Vec::new());
    }
    let (ok_entries, bad_entries): (Vec<String>, Vec<String>) =
        list.iter().cloned().partition(|e| choices.contains(e));
    if bad_entries.is_empty() {
        Ok(ok_entries)
    } else {
        Err(bad_entries)
    }
}

fn handle_multi_select_input(
    provided_value: Option<String>,
    var_name: &str,
    entry: &ArrayEntry,
    prompt: &Prompt,
) -> Result<String> {
    let val = match provided_value {
        // value is just provided
        Some(value) => value,
        // no value is provided so we have to be smarter
        None => {
            let mut ms = cliclack::multiselect(&prompt.raw).required(false);
            for choice in &entry.choices {
                let is_default = entry
                    .default
                    .as_ref()
                    .is_some_and(|d| d.contains(choice));
                let hint = if is_default { "default" } else { "" };
                ms = ms.item(choice.clone(), choice, hint);
            }
            if let Some(defaults) = &entry.default {
                let initial: Vec<String> = defaults
                    .iter()
                    .filter(|d| entry.choices.contains(d))
                    .cloned()
                    .collect();
                ms = ms.initial_values(initial);
            }

            let chosen: Vec<String> = ms.interact()?;
            chosen.join(LIST_SEP)
        }
    };

    match check_provided_selections(&val, &entry.choices) {
        Ok(s) => Ok(s.join(LIST_SEP)),
        Err(s) => {
            let err_string = if s.len() > 1 {
                format!("are not valid values for {var_name}")
            } else {
                format!("is not a valid value for {var_name}")
            };

            bail!(
                "{} {} \"{}\" {}",
                emoji::WARN,
                style("Sorry,").bold().red(),
                style(&s.join(LIST_SEP)).bold().yellow(),
                style(err_string).bold().red(),
            )
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
            let chosen: bool = cliclack::confirm(&prompt.raw)
                .initial_value(default.unwrap_or(false))
                .interact()?;
            Ok(chosen.to_string())
        }
    }
}

fn open_editor(initial_content: &str) -> Result<Option<String>> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile()?;
    write!(tmp, "{}", initial_content)?;
    tmp.flush()?;

    let path = tmp.path().to_owned();
    let status = std::process::Command::new(&editor).arg(&path).status()?;

    if !status.success() {
        return Ok(None);
    }

    let content = fs_err::read_to_string(&path)?;
    if content == initial_content {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}
