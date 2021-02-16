use crate::{
    app_config::{app_config_path, AppConfig, FavoriteConfig},
    info, Args,
};
use anyhow::{Context, Result};
use console::style;

pub(crate) fn list_favorites(args: &Args) -> Result<()> {
    let path = &app_config_path(&args.config)?;
    println!("Listing favorites defined in: {}", path.as_path().display());
    let app_config = AppConfig::from_path(path)?;

    let data = {
        let mut d = app_config
            .favorites
            .iter()
            .filter(|(key, _)| args.favorite.as_ref().map_or(true, |f| key.starts_with(f)))
            .collect::<Vec<(&String, &FavoriteConfig)>>();
        d.sort_by_key(|(key, _)| key.to_string());
        d
    };

    println!("Possible favorites:");
    let longest_key = data.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let longest_key = ((longest_key + 5) / 4) * 4;
    data.iter().for_each(|(key, conf)| {
        println!(
            "{}:{}{}",
            key,
            " ".repeat(longest_key - key.len()),
            conf.description.as_ref().cloned().unwrap_or_default()
        );
    });

    Ok(())
}

pub(crate) fn resolve_favorite(args: &mut Args) -> Result<()> {
    if args.git.is_some() {
        return Ok(());
    }

    let favorite_name = args
        .favorite
        .as_ref()
        .with_context(|| "Please specify either --git option, or a predefined favorite")?;

    let app_config_path = app_config_path(&args.config)?;
    let app_config = AppConfig::from_path(app_config_path.as_path())?;

    let (git, branch, template_values_file) = app_config
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
                    None,
                )
            },
            |f| {
                (
                    f.git.clone(),
                    args.branch.as_ref().or_else(|| f.branch.as_ref()).cloned(),
                    f.template_values.clone(),
                )
            },
        );

    let args_template_values_file = args.template_values_file.take().unwrap_or_default();
    let v: Vec<String> = args_template_values_file
        .into_iter()
        .chain(template_values_file.into_iter())
        .collect();

    args.git = git;
    args.branch = branch;
    args.template_values_file = Some(v);
    Ok(())
}
