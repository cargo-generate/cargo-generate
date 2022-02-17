//! Module dealing with <favorite> arg passed to cargo-generate

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    app_config::{AppConfig, FavoriteConfig},
    emoji, Args,
};
use anyhow::Result;
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

// Location of template before copy it into temp directory
pub struct SourceTemplate {
    template_location: TemplateLocation,
    subfolder: Option<String>,
    template_values: HashMap<String, toml::Value>,
}

impl SourceTemplate {
    fn new(
        template_location: impl Into<TemplateLocation>,
        subfolder: Option<String>,
        default_values: HashMap<String, toml::Value>,
    ) -> Self {
        Self {
            template_location: template_location.into(),
            subfolder,
            template_values: default_values,
        }
    }

    pub fn try_from_args_and_config(app_config: &AppConfig, args: &Args) -> Self {
        // --git and --path can not be set at the same time
        assert!(args.git.is_none() || args.path.is_none());
        let mut default_values = app_config.values.clone().unwrap_or_default();

        // --git
        if let Some(git_url) = &args.git {
            assert!(args.subfolder.is_none());

            let git_user_in = GitUserInput::with_git_url_and_args(git_url, args);
            return Self::new(git_user_in, args.favorite.clone(), default_values);
        }

        // --path
        if let Some(path) = &args.path {
            assert!(args.subfolder.is_none());

            return Self::new(path, args.favorite.clone(), default_values);
        }

        // check if favorite is favorite configuration
        assert!(args.favorite.is_some());
        let fav_name = args.favorite.as_ref().unwrap();

        if let Some(fav_cfg) = app_config.get_favorite_cfg(fav_name) {
            assert!(fav_cfg.git.is_none() || fav_cfg.path.is_none());

            let temp_location = if let Some(git_url) = &fav_cfg.git {
                let branch = args.branch.clone().or_else(|| fav_cfg.branch.clone());
                let git_user_input = GitUserInput::new(
                    git_url,
                    branch,
                    args.ssh_identity.clone(),
                    args.force_git_init,
                );

                TemplateLocation::from(git_user_input)
            } else if let Some(path) = &fav_cfg.path {
                TemplateLocation::from(path)
            } else {
                unreachable!();
            };

            if let Some(fav_default_values) = &fav_cfg.values {
                default_values.extend(fav_default_values.clone());
            }

            return Self::new(
                temp_location,
                args.subfolder.clone().or_else(|| fav_cfg.subfolder.clone()),
                default_values,
            );
        }

        // there is no specified favorite in configuration
        // this part try to guess what user wanted in order:
        // 1. look for abbrevations like gh:, gl: etc.
        // 2. check if template directory exist
        // 3. check if the input is in form org/repo<> (map to github)
        // 4. assume user wanted use --git

        let tl = if let Some(git_url) = abbreviated_git_url_to_full_remote(fav_name) {
            TemplateLocation::from(GitUserInput::with_git_url_and_args(&git_url, args))
        } else if let Some(path) = local_path(fav_name) {
            TemplateLocation::Path(path)
        } else if let Some(git_url) = abbreviated_github(fav_name) {
            TemplateLocation::from(GitUserInput::with_git_url_and_args(&git_url, args))
        } else {
            TemplateLocation::from(GitUserInput::with_git_url_and_args(fav_name, args))
        };

        Self::new(tl, args.subfolder.clone(), default_values)
    }

    pub fn location(&self) -> &TemplateLocation {
        &self.template_location
    }

    pub fn subfolder(&self) -> Option<&str> {
        self.subfolder.as_deref()
    }

    pub fn template_values(&self) -> &HashMap<String, toml::Value> {
        &self.template_values
    }

    pub fn template_values_mut(&mut self) -> &mut HashMap<String, toml::Value> {
        &mut self.template_values
    }
}

// favorite can be in form with abbrevation what means that input is git repositoru
pub fn abbreviated_git_url_to_full_remote(git: impl AsRef<str>) -> Option<String> {
    let git = git.as_ref();
    if git.len() >= 3 {
        match &git[..3] {
            "gl:" => Some(format!("https://gitlab.com/{}.git", &git[3..])),
            "bb:" => Some(format!("https://bitbucket.org/{}.git", &git[3..])),
            "gh:" => Some(format!("https://github.com/{}.git", &git[3..])),
            _ => None,
        }
    } else {
        None
    }
}

// favorite can be in form of org/repo what should be parsed as github.com
pub fn abbreviated_github(fav: &str) -> Option<String> {
    let count_slash = fav.chars().filter(|c| *c == '/').count();
    (count_slash == 1).then(|| format!("https://github.com/{fav}"))
}

pub fn local_path(fav: &str) -> Option<PathBuf> {
    let path = PathBuf::from(fav);
    (path.exists() && path.is_dir()).then(|| path)
}

// Represent user input to locate template
pub struct GitUserInput {
    url: String,
    branch: Option<String>,
    identity: Option<PathBuf>,
    force_init: bool,
}

impl GitUserInput {
    fn new(url: &str, branch: Option<String>, identity: Option<PathBuf>, force_init: bool) -> Self {
        Self {
            url: url.to_owned(),
            branch,
            identity,
            force_init,
        }
    }

    // when git was used as abbreviation but other flags still could be passed
    fn with_git_url_and_args(url: &str, args: &Args) -> Self {
        Self::new(
            url,
            args.branch.clone(),
            args.ssh_identity.clone(),
            args.force_git_init,
        )
    }

    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn identity(&self) -> Option<&Path> {
        self.identity.as_deref()
    }
}

pub enum TemplateLocation {
    Git(GitUserInput),
    Path(PathBuf),
}

impl From<GitUserInput> for TemplateLocation {
    fn from(source: GitUserInput) -> Self {
        Self::Git(source)
    }
}

impl<T> From<T> for TemplateLocation
where
    T: AsRef<Path>,
{
    fn from(source: T) -> Self {
        Self::Path(PathBuf::from(source.as_ref()))
    }
}
