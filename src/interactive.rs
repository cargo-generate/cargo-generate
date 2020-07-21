use crate::emoji;
use console::style;
use dialoguer::Input;
use quicli::prelude::Error;

pub fn name() -> Result<String, Error> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let name = loop {
        let name = Input::new(&format!(
            "{} {}",
            emoji::SHRUG,
            style("Project Name").bold()
        ))
        .interact()?;
        if valid_ident.is_match(&name) {
            break name;
        } else {
            eprintln!(
                "{} {} \"{}\" {}",
                emoji::WARN,
                style("Sorry,").bold().red(),
                style(&name).bold().yellow(),
                style("is not a valid crate name").bold().red()
            );
        }
    };
    Ok(name)
}
