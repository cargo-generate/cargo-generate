mod authors;
mod crate_type;
mod os_arch;
mod project_name;

use crate::{config::ConfigValues, emoji, Args};

use anyhow::Result;
use console::style;
use regex::Regex;
use std::{collections::HashMap, fs, path::Path};
use toml::Value;

pub(crate) use authors::{get_authors, Authors};
pub(crate) use crate_type::CrateType;
pub(crate) use os_arch::get_os_arch;
pub(crate) use project_name::ProjectName;

pub(crate) fn resolve_template_values(args: &Args) -> Result<HashMap<String, Value>> {
    let mut template_values = args
        .template_values_file
        .as_ref()
        .map(|p| Path::new(p))
        .map_or(Ok(Default::default()), |path| {
            read_template_values_file(path)
        })?;

    add_cli_defined_values(&mut template_values, args)?;

    Ok(template_values)
}

fn read_template_values_file(path: &Path) -> Result<HashMap<String, Value>> {
    match fs::read_to_string(path) {
        Ok(ref contents) => toml::from_str::<ConfigValues>(contents)
            .map(|v| v.values)
            .map_err(|e| e.into()),
        Err(e) => anyhow::bail!(
            "{} {} {}",
            emoji::ERROR,
            style("Values File Error:").bold().red(),
            style(e).bold().red(),
        ),
    }
}

fn add_cli_defined_values(template_values: &mut HashMap<String, Value>, args: &Args) -> Result<()> {
    let regex = Regex::new(r"(\w+)=(.+)").unwrap();

    args.define
        .iter()
        .try_fold(template_values, |template_values, d| {
            match regex.captures(d) {
                Some(cap) => {
                    let k = cap.get(1).unwrap().as_str().to_string();
                    let v = cap.get(2).unwrap().as_str().to_string();
                    println!("{} => '{}'", k, v);
                    template_values.insert(k, Value::from(v));
                    Ok(template_values)
                }
                None => Err(anyhow::anyhow!(
                    "{} {} {}",
                    emoji::ERROR,
                    style("Failed to parse value:").bold().red(),
                    style(d).bold().red(),
                )),
            }
        })?;
    Ok(())
}
