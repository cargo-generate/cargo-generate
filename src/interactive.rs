use console::style;
use dialoguer::Input;
use emoji;
use heck::KebabCase;
use quicli::prelude::Error;
use regex;

pub fn name() -> Result<String, Error> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let name = loop {
        let name = Input::new(&format!(
            "{} {}",
            emoji::SHRUG,
            style("Project Name").bold()
        )).interact()?;
        if valid_ident.is_match(&name) {

            println!(
                "{} {} `{}`{}",
                emoji::WRENCH,
                style("Creating project called").bold(),
                style(&name.to_kebab_case()).bold().yellow(),
                style("...").bold()
            );
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
