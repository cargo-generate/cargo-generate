use crate::{
    emoji,
    project_variables::{StringEntry, TemplateSlots, VarInfo},
};
use anyhow::Result;
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use liquid_core::Value;
use std::ops::Index;

pub(crate) fn name() -> Result<String> {
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
    prompt_for_variable(&project_var)
}

pub(crate) fn user_question(prompt: &str, default: &Option<String>) -> Result<String> {
    let mut i = Input::<String>::new();
    i.with_prompt(prompt.to_string());
    if let Some(s) = default {
        i.default(s.to_owned());
    }
    i.interact().map_err(Into::<anyhow::Error>::into)
}

fn extract_default(variable: &VarInfo) -> Option<String> {
    match variable {
        VarInfo::Bool {
            default: Some(d), ..
        } => Some(if *d { "true".into() } else { "false".into() }),
        VarInfo::String { entry } => match entry.as_ref() {
            StringEntry {
                default: Some(d), ..
            } => Some(d.into()),
            _ => None,
        },
        _ => None,
    }
}

fn prompt_for_variable(variable: &TemplateSlots) -> Result<String> {
    let prompt = format!("{} {}", emoji::SHRUG, style(&variable.prompt).bold(),);

    if let VarInfo::String { entry } = &variable.var_info {
        if let Some(choices) = &entry.choices {
            use dialoguer::Select;

            let default = if let Some(default) = &entry.default {
                choices.binary_search(default).unwrap_or(0)
            } else {
                0
            };
            let chosen = Select::with_theme(&ColorfulTheme::default())
                .paged(choices.len() > Term::stdout().size().0 as usize)
                .items(choices)
                .with_prompt(&prompt)
                .default(default)
                .interact()?;

            return Ok(choices.index(chosen).to_string());
        }
    }

    let prompt = format!("{} {}", prompt, choice_options(&variable.var_info));
    loop {
        let default = extract_default(&variable.var_info);
        let user_entry = user_question(prompt.as_str(), &default)?;

        if is_valid_variable_value(&user_entry, &variable.var_info) {
            return Ok(user_entry);
        } else {
            eprintln!(
                "{} {} \"{}\" {}",
                emoji::WARN,
                style("Sorry,").bold().red(),
                style(&user_entry).bold().yellow(),
                style(format!("is not a valid value for {}", variable.var_name))
                    .bold()
                    .red()
            );
        }
    }
}

pub(super) fn variable(variable: &TemplateSlots) -> Result<Value> {
    let user_input = prompt_for_variable(variable)?;
    into_value(user_input, &variable.var_info)
}

fn is_valid_variable_value(user_entry: &str, var_info: &VarInfo) -> bool {
    match var_info {
        VarInfo::Bool { .. } => user_entry.parse::<bool>().is_ok(),
        VarInfo::String { entry } => match entry.as_ref() {
            StringEntry {
                choices: Some(options),
                regex: Some(reg),
                ..
            } => options.iter().any(|x| x == user_entry) && reg.is_match(user_entry),
            StringEntry {
                choices: Some(options),
                regex: None,
                ..
            } => options.iter().any(|x| x == user_entry),
            StringEntry {
                choices: None,
                regex: Some(reg),
                ..
            } => reg.is_match(user_entry),
            StringEntry {
                choices: None,
                regex: None,
                ..
            } => true,
        },
    }
}

fn into_value(user_entry: String, var_info: &VarInfo) -> Result<Value> {
    match var_info {
        VarInfo::Bool { .. } => {
            let as_bool = user_entry.parse::<bool>()?; // this shouldn't fail if checked before
            Ok(Value::Scalar(as_bool.into()))
        }
        VarInfo::String { .. } => Ok(Value::Scalar(user_entry.into())),
    }
}

fn choice_options(var_info: &VarInfo) -> String {
    match var_info {
        VarInfo::Bool { default: None } => "[true, false]".to_string(),
        VarInfo::Bool { default: Some(d) } => {
            format!("[true, false] [default: {}]", style(d).bold())
        }
        VarInfo::String { entry } => match entry.as_ref() {
            StringEntry {
                choices: Some(ref cs),
                default: None,
                ..
            } => format!("[{}]", cs.join(", ")),
            StringEntry {
                choices: Some(ref cs),
                default: Some(ref d),
                ..
            } => {
                format!("[{}] [default: {}]", cs.join(", "), style(d).bold())
            }
            StringEntry {
                choices: None,
                default: Some(ref d),
                ..
            } => {
                format!("[default: {}]", style(d).bold())
            }
            _ => "".to_string(),
        },
    }
}
