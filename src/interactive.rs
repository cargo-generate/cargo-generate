use crate::emoji;
use anyhow::Result;
use console::style;
use dialoguer::Input;

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
