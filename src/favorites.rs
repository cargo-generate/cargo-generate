use crate::{
    app_config::{AppConfig, FavoriteConfig},
    emoji, info, Args,
};
use anyhow::{anyhow, Result};
use console::style;

pub(crate) fn list_favorites(app_config: &AppConfig, args: &Args) -> Result<()> {
    let data = {
        let mut d = app_config
            .favorites
            .iter()
            .filter(|(key, _)| args.favorite.as_ref().map_or(true, |f| key.starts_with(f)))
            .collect::<Vec<(&String, &FavoriteConfig)>>();
        d.sort_by_key(|(key, _)| key.to_string());
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

pub(crate) fn resolve_favorite_args(app_config: &AppConfig, args: &mut Args) -> Result<()> {
    if args.git.is_some() {
        args.subfolder = args.favorite.take();
        return Ok(());
    }

    let favorite_name = args
        .favorite
        .as_ref()
        .ok_or_else(|| anyhow!("Please specify either --git option, or a predefined favorite"))?;

    let (git, branch, subfolder) = app_config
        .favorites
        .get(favorite_name.as_str())
        .map_or_else(
            || {
                info!(
                    "Favorite {} not found in config, using it as a git repo url",
                    style(&favorite_name).bold()
                );
                (
                    Some(favorite_name.clone()),
                    args.branch.as_ref().cloned(),
                    args.subfolder.clone(),
                )
            },
            |f| {
                (
                    f.git.clone(),
                    args.branch.as_ref().or_else(|| f.branch.as_ref()).cloned(),
                    args.subfolder
                        .as_ref()
                        .or_else(|| f.subfolder.as_ref())
                        .cloned(),
                )
            },
        );

    args.git = git;
    args.branch = branch;
    args.subfolder = subfolder;

    Ok(())
}
