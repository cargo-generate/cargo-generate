use std::fmt::Display;

use anyhow::anyhow;
use console::style;
use liquid::ValueView;

use crate::{
    emoji, interactive, template::LiquidObjectResource, user_parsed_input::UserParsedInput,
};
use log::warn;

#[derive(Debug)]
pub struct ProjectNameInput(String);

impl TryFrom<(&LiquidObjectResource, &UserParsedInput)> for ProjectNameInput {
    type Error = anyhow::Error;

    fn try_from(
        (liquid_object, user_parsed_input): (&LiquidObjectResource, &UserParsedInput),
    ) -> Result<Self, Self::Error> {
        let name = liquid_object
            .lock()
            .unwrap()
            .borrow()
            .get("project-name")
            .map(|v| {
                let v = v.as_scalar().to_kstr().into_string();
                if let Some(n) = user_parsed_input.name() {
                    if n != v {
                        warn!(
                            "{} {} `{}` {} `{}`{}",
                            emoji::WARN,
                            style("Project name changed by template, from").bold(),
                            style(n).bold().yellow(),
                            style("to").bold(),
                            style(&v).bold().green(),
                            style("...").bold()
                        );
                    }
                }
                v
            })
            .or_else(|| user_parsed_input.name().map(String::from));

        match name {
            Some(name) => Ok(Self(name)),
            None => {
                match std::env::var("CARGO_GENERATE_VALUE_PROJECT_NAME") {
                    Ok(name) => Ok(Self(name)),
                    Err(_) if !user_parsed_input.silent() => Ok(Self(interactive::name()?)),
                    Err(_) => Err(anyhow!(
                        "{} {} {}",
                        emoji::ERROR,
                        style("Project Name Error:").bold().red(),
                        style("Option `--silent` provided, but project name was not set. Please use `--name`.")
                            .bold()
                            .red(),
                    )),
                }
            }
        }
    }
}

impl AsRef<str> for ProjectNameInput {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for ProjectNameInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
