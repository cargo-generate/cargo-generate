use std::collections::HashMap;

use crate::{
    app_config::{AppConfig, FavoriteConfig},
    emoji, warn, Args,
};
use anyhow::{anyhow, Result};
use console::style;

pub fn list_favorites(app_config: &AppConfig, args: &Args) -> Result<()> {
    let data = {
        let mut d = app_config
            .favorites
            .as_ref()
            .map(|h| {
                h.iter()
                    .filter(|(key, _)| args.favorite.as_ref().map_or(true, |f| key.starts_with(f)))
                    .collect::<Vec<(&String, &FavoriteConfig)>>()
            })
            .unwrap_or_default();
        d.sort_by_key(|(key, _)| (*key).to_string());
        d
    };

    if data.is_empty() {
        println!(
            "{} {}",
            emoji::WARN,
            style("No favorites defined").bold().red()
        );
        return Ok(());
    }

    println!("{} {}", emoji::WRENCH, style("Possible favorites:").bold());
    let longest_key = data.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let longest_key = ((longest_key + 5) / 4) * 4;
    data.iter().for_each(|(key, conf)| {
        println!(
            "    {} {}:{}{}",
            emoji::DIAMOND,
            style(key).bold(),
            " ".repeat(longest_key - key.len()),
            conf.description.as_ref().cloned().unwrap_or_default()
        );
    });
    println!("{} {}", emoji::SPARKLE, style("Done").bold().green());

    Ok(())
}

pub fn resolve_favorite_args_and_default_values(
    app_config: &AppConfig,
    args: &mut Args,
) -> Result<Option<HashMap<String, toml::Value>>> {
    if args.git.is_some() {
        args.subfolder = args.favorite.take();
        return Ok(app_config.values.clone());
    }

    if args.path.is_some() {
        return Ok(app_config.values.clone());
    }

    let favorite_name = args
        .favorite
        .as_ref()
        .ok_or_else(|| anyhow!("Please specify either --git option, or a predefined favorite"))?;

    let (values, git, branch, subfolder, path) = app_config
        .favorites
        .as_ref()
        .and_then(|f| f.get(favorite_name.as_str()))
        .map_or_else(
            || {
                warn!(
                    "Favorite {} not found in config, using it as a git repo url",
                    style(&favorite_name).bold()
                );
                (
                    None,
                    Some(favorite_name.clone()),
                    args.branch.as_ref().cloned(),
                    args.subfolder.clone(),
                    None,
                )
            },
            |f| {
                let values = match app_config.values.clone() {
                    Some(mut values) => {
                        values.extend(f.values.clone().unwrap_or_default());
                        Some(values)
                    }
                    None => f.values.clone(),
                };

                (
                    values,
                    f.git.clone(),
                    args.branch.as_ref().or_else(|| f.branch.as_ref()).cloned(),
                    args.subfolder
                        .as_ref()
                        .or_else(|| f.subfolder.as_ref())
                        .cloned(),
                    f.path.clone(),
                )
            },
        );

    args.git = git;
    args.branch = branch;
    args.subfolder = subfolder;
    args.path = path;

    Ok(values)
}
