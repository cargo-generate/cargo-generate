use dialoguer::Input;
use ident_case;
use quicli::prelude::Error;
use regex;

pub fn name() -> Result<String, Error> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let name = loop {
        let name = Input::new("The project's name is").interact()?;
        if valid_ident.is_match(&name) {
            let name = ident_case::RenameRule::KebabCase.apply_to_field(&name);
            println!("Nice, I'll call your project `{}`", name);
            break name;
        } else {
            eprintln!("Sorry, that is not a valid crate name :(");
        }
    };
    Ok(name)
}
