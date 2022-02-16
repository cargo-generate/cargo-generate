//! Module dealing with <favorite> arg passed to cargo-generate

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

// Location of template before copy it into temp directory
pub struct SourceTemplate {
    template_location: TemplateLocation,
    subfolder: Option<String>,
}

impl SourceTemplate {
    fn new(template_location: impl Into<TemplateLocation>, subfolder: Option<String>) -> Self {
        Self {
            template_location: template_location.into(),
            subfolder,
        }
    }

    pub fn try_from_args_and_config(app_config: &AppConfig, args: &Args) -> Self {
        // --git and --path can not be set at the same time
        assert!(args.git.is_none() || args.path.is_none());

        // --git
        if let Some(git_user_in) = GitUserInput::try_from_args(args) {
            assert!(args.subfolder.is_none());

            return Self::new(git_user_in, args.favorite.clone());
        }

        // --path
        if let Some(path) = &args.path {
            assert!(args.subfolder.is_none());

            return Self::new(path, args.favorite.clone());
        }

        // check if favorite is favorite configuration
        assert!(args.favorite.is_some());
        let fav_name = args.favorite.as_ref().unwrap();

        if let Some(fav_cfg) = app_config.get_favorite_cfg(fav_name) {
            assert!(fav_cfg.git.is_none() || fav_cfg.path.is_none());

            let temp_location = if let Some(git_url) = &fav_cfg.git {
                let git_user_input = GitUserInput {
                    url: git_url.clone(),
                    branch: args.branch.clone().or_else(|| fav_cfg.branch.clone()),
                    identity: args.ssh_identity.clone(),
                    force_init: args.force_git_init,
                };

                TemplateLocation::from(git_user_input)
            } else if let Some(path) = &fav_cfg.path {
                TemplateLocation::from(path)
            } else {
                unreachable!();
            };

            return Self::new(
                temp_location,
                args.subfolder.clone().or_else(|| fav_cfg.subfolder.clone()),
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

        Self::new(tl, args.subfolder.clone())
    }

    pub fn location(&self) -> &TemplateLocation {
        &self.template_location
    }

    pub fn subfolder(&self) -> Option<&str> {
        self.subfolder.as_deref()
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
    (count_slash == 1).then(||
        format!("https://github.com/{fav}"))
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
    // when git was used as abbreviation but other flags still could be passed
    fn with_git_url_and_args(url: &str, args: &Args) -> Self {
        Self {
            url: url.to_owned(),
            branch: args.branch.clone(),
            identity: args.ssh_identity.clone(),
            force_init: args.force_git_init,
        }
    }

    fn try_from_args(args: &Args) -> Option<Self> {
        Some(Self::with_git_url_and_args(args.git.as_deref()?, args))
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
                    "Favorite {} not found in config, using it as path or git repo url",
                    style(&favorite_name).bold()
                );

                let what_is_fav = GitOrPath::from_favorite_name(favorite_name);
                (
                    None,
                    what_is_fav.git(),
                    args.branch.as_ref().cloned(),
                    args.subfolder.clone(),
                    what_is_fav.path_with_subfolder(args.subfolder.as_ref()),
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

/// Distinguish for favorite to be remote git or local path
enum GitOrPath {
    Git(String),
    Path(PathBuf),
}

impl GitOrPath {
    fn from_favorite_name(favorite: &str) -> Self {
        let maybe_path = Path::new(favorite);
        if maybe_path.exists() && maybe_path.is_dir() {
            println!("exist path and dir {:?}", maybe_path);
            Self::Path(maybe_path.into())
        } else {
            println!("git: {:?}", favorite);
            Self::Git(favorite.into())
        }
    }

    fn git(&self) -> Option<String> {
        match self {
            Self::Git(v) => Some(v.clone()),
            Self::Path(_) => None,
        }
    }

    fn path(&self) -> Option<PathBuf> {
        match self {
            Self::Git(_) => None,
            Self::Path(path) => Some(path.clone()),
        }
    }

    fn path_with_subfolder(&self, subfolder: Option<&String>) -> Option<PathBuf> {
        self.path().map(|path| {
            if let Some(subfolder) = subfolder {
                path.join(subfolder)
            } else {
                path
            }
        })
    }
}
