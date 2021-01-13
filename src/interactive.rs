use crate::{
    emoji,
    project_variables::{StringEntry, TemplateSlots, VarInfo},
};
use anyhow::Result;
use console::style;
use dialoguer::Input;
use liquid_core::Value;

pub(crate) fn name() -> Result<String> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    loop {
        let name: String = Input::new()
            .with_prompt(format!("{} {}", emoji::SHRUG, style("Project Name").bold()))
            .interact()?;

        if valid_ident.is_match(&name) {
            return Ok(name);
        } else {
            eprintln!(
                "{} {} \"{}\" {}",
                emoji::WARN,
                style("Sorry,").bold().red(),
                style(&name).bold().yellow(),
                style("is not a valid crate name").bold().red()
            );
        }
    }
}

pub(crate) fn user_question(prompt: &str) -> Result<String> {
    Input::<String>::new()
        .with_prompt(prompt.to_string())
        .interact()
        .map_err(Into::<anyhow::Error>::into)
}

pub(crate) fn variable<F: Fn(&str) -> Result<String>>(
    variable: &TemplateSlots,
    var_pooler: F,
) -> Result<Value> {
    let prompt = format!(
        "{} {} {}",
        emoji::SHRUG,
        style(&variable.prompt).bold(),
        choice_options(&variable.var_info)
    );
    loop {
        let user_entry = var_pooler(&prompt)?;

        if is_valid_variable_value(&user_entry, &variable.var_info) {
            return into_value(user_entry, &variable.var_info);
        } else {
            eprintln!(
                "{} {} \"{}\" {}",
                emoji::WARN,
                style("Sorry,").bold().red(),
                style(&user_entry).bold().yellow(),
                style("is not a valid value").bold().red()
            );
        }
    }
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
