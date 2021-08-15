use anyhow::{Context, Result};
use console::style;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::emoji;

pub(crate) const CONFIG_FILE_NAME: &str = "cargo-generate.toml";

#[derive(Deserialize)]
pub(crate) struct AppConfig {
    pub favorites: HashMap<String, FavoriteConfig>,
}

#[derive(Deserialize, Default)]
pub(crate) struct FavoriteConfig {
    pub description: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub subfolder: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            favorites: HashMap::new(),
        }
    }
}

impl AppConfig {
    pub(crate) fn from_path(path: &Path) -> Result<AppConfig> {
        if path.exists() {
            let cfg = fs::read_to_string(path)?;
            Ok(if cfg.trim().is_empty() {
                AppConfig::default()
            } else {
                println!(
                    "{} {} {}",
                    emoji::INFO,
                    style("Using application config:").bold(),
                    style(path.display()).underlined()
                );
                toml::from_str(&cfg)?
            })
        } else {
            crate::info!(
                "Unable to load config file: {}",
                style(path.display()).bold().yellow()
            );
            Ok(Default::default())
        }
    }
}

pub(crate) fn app_config_path(path: &Option<PathBuf>) -> Result<PathBuf> {
    path.clone()
        .or_else(|| {
            home::cargo_home().map_or(None, |home| {
                let preferred_path = home.join(CONFIG_FILE_NAME);
                match preferred_path.exists() {
                    true => Some(preferred_path),
                    false => {
                        let without_extension = preferred_path.with_extension("");
                        match without_extension.exists() {
                            true => Some(without_extension),
                            false => Some(preferred_path),
                        }
                    }
                }
            })
        })
        .with_context(|| {
            format!(
                r#"
Unable to resolve path for configuration file.
Use --config option, or place {} in $CARGO_HOME."#,
                CONFIG_FILE_NAME
            )
        })
}
