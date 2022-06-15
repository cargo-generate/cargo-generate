//! Module dealing with <favorite> arg passed to cargo-generate

use crate::{
    app_config::{AppConfig, FavoriteConfig},
    emoji, GenerateArgs,
};
use anyhow::Result;
use console::style;

pub fn list_favorites(app_config: &AppConfig, args: &GenerateArgs) -> Result<()> {
    let data = {
        let mut d = app_config
            .favorites
            .as_ref()
            .map(|h| {
                h.iter()
                    .filter(|(key, _)| {
                        args.template_path
                            .auto_path()
                            .map_or(true, |f| key.starts_with(f))
                    })
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
