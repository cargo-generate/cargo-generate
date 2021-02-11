use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub(crate) const CONFIG_FILE_NAME: &str = "cargo-generate";

#[derive(Deserialize)]
pub(crate) struct AppConfig {
    pub favorites: HashMap<String, FavoriteConfig>,
}

#[derive(Deserialize)]
pub(crate) struct FavoriteConfig {
    pub description: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
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
                toml::from_str(&cfg)?
            })
        } else {
            Ok(Default::default())
        }
    }
}

pub(crate) fn app_config_path(path: &Option<PathBuf>) -> Result<PathBuf> {
    path.clone()
        .or_else(|| home::cargo_home().map_or(None, |h| Some(h.join(CONFIG_FILE_NAME))))
        .with_context(|| {
            format!(
                r#"
Unable to resolve path for configuration file.
Use --config option, or place {} in $CARGO_HOME."#,
                CONFIG_FILE_NAME
            )
        })
}
