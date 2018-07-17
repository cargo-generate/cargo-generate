use emoji;
use dialoguer::Input;
use ident_case;
use quicli::prelude::Error;
use regex;

pub fn name() -> Result<String, Error> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let name = loop {
        let name = Input::new(&format!("{} Project Name", emoji::SHRUG)).interact()?;
        if valid_ident.is_match(&name) {
            let name = ident_case::RenameRule::KebabCase.apply_to_field(&name);
            println!("{} Creating project called `{}`...", emoji::WRENCH, name);
            break name;
        } else {
            eprintln!("{} Sorry, \"{}\" is not a valid crate name", name, emoji::WARN);
        }
    };
    Ok(name)
}
