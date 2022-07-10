use crate::{
    emoji,
    project_variables::{StringEntry, TemplateSlots, VarInfo},
};
use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use liquid_core::Value;
use std::ops::Index;

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
    prompt_for_variable(&project_var)
}

pub fn user_question(prompt: &str, default: &Option<String>) -> Result<String> {
    let mut i = Input::<String>::new();
    i.with_prompt(prompt.to_string());
    if let Some(s) = default {
        i.default(s.to_owned());
    }
    i.interact().map_err(Into::<anyhow::Error>::into)
}

pub fn prompt_for_variable(variable: &TemplateSlots) -> Result<String> {
    use dialoguer::Select;

    let prompt = format!("{} {}", emoji::SHRUG, style(&variable.prompt).bold(),);

    match &variable.var_info {
        VarInfo::Bool { default } => {
            let choices = [false.to_string(), true.to_string()];
            let chosen = Select::with_theme(&ColorfulTheme::default())
                .items(&choices)
                .with_prompt(&prompt)
                .default(if default.unwrap_or(false) { 1 } else { 0 })
                .interact()?;

            Ok(choices.index(chosen).to_string())
        }
        VarInfo::String { entry } => match &entry.choices {
            Some(choices) => {
                let default = entry
                    .default
                    .as_ref()
                    .map_or(0, |default| choices.binary_search(default).unwrap_or(0));
                let chosen = Select::with_theme(&ColorfulTheme::default())
                    .items(choices)
                    .with_prompt(&prompt)
                    .default(default)
                    .interact()?;

                Ok(choices.index(chosen).to_string())
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
                            style(format!("is not a valid value for {}", variable.var_name))
                                .bold()
                                .red()
                        );
                    },
                    None => Ok(user_question(prompt.as_str(), &default)?),
                }
            }
        },
    }
}

pub fn variable(variable: &TemplateSlots, provided_value: Option<&impl ToString>) -> Result<Value> {
    let user_input = provided_value
        .map(|v| Ok(v.to_string()))
        .unwrap_or_else(|| prompt_for_variable(variable))?;
    into_value(user_input, &variable.var_info)
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
