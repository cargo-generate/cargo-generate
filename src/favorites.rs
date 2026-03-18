//! Module dealing with &lt;favorite&gt; arg passed to cargo-generate

use crate::{
    app_config::{app_config_path, AppConfig, FavoriteConfig},
    GenerateArgs,
};
use anyhow::Result;

pub fn list_favorites(args: &GenerateArgs) -> Result<()> {
    let app_config: AppConfig = app_config_path(&args.config)?.as_path().try_into()?;

    let data = {
        let mut d = app_config
            .favorites
            .as_ref()
            .map(|h| {
                h.iter()
                    .filter(|(key, _)| {
                        args.template_path
                            .auto_path()
                            .is_none_or(|f| key.starts_with(f.as_ref()))
                    })
                    .collect::<Vec<(&String, &FavoriteConfig)>>()
            })
            .unwrap_or_default();
        d.sort_by_key(|(key, _)| (*key).to_string());
        d
    };

    if data.is_empty() {
        cliclack::log::warning("No favorites defined")?;
        return Ok(());
    }

    let longest_key = data.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let longest_key = ((longest_key + 5) / 4) * 4;
    let lines: Vec<String> = data
        .iter()
        .map(|(key, conf)| {
            format!(
                "  {}{}{}",
                key,
                " ".repeat(longest_key - key.len()),
                conf.description
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| "no description".into())
            )
        })
        .collect();
    cliclack::note("Favorites", lines.join("\n"))?;

    Ok(())
}
