use crate::{
    app_config::{app_config_path, AppConfig, FavoriteConfig},
    Args,
};
use anyhow::{Context, Result};

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
    if args.git.is_some() || args.dir.is_some() {
        return Ok(());
    }

    let favorite_name = args
        .favorite
        .as_ref()
        .context("Please specify either --git, --dir, or a predefined favorite")?;

    let app_config_path = app_config_path(&args.config)?;
    let app_config = AppConfig::from_path(app_config_path.as_path())?;
    let favorite = app_config
        .favorites
        .get(favorite_name.as_str())
        .with_context(|| {
            format!(
                "Unknown favorite: {}\nFavorites must be defined in the configuration file at: {}",
                favorite_name,
                app_config_path.display()
            )
        })?;

    args.git = favorite.git.clone();
    args.dir = args.dir.as_ref().or_else(|| favorite.dir.as_ref()).cloned();
    args.branch = args
        .branch
        .as_ref()
        .or_else(|| favorite.branch.as_ref())
        .cloned();

    Ok(())
}
