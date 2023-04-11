use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::bail;
use console::style;
use heck::ToKebabCase;

use crate::{emoji, user_parsed_input::UserParsedInput};
use log::warn;

use super::project_name_input::ProjectNameInput;

/// Stores user inputted name and provides convenience methods
/// for handling casing.
pub struct ProjectDir(PathBuf);

impl AsRef<Path> for ProjectDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl Display for ProjectDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

impl TryFrom<(&ProjectNameInput, &UserParsedInput)> for ProjectDir {
    type Error = anyhow::Error;

    fn try_from(
        (project_name_input, user_parsed_input): (&ProjectNameInput, &UserParsedInput),
    ) -> Result<Self, Self::Error> {
        let base_path = user_parsed_input.destination();

        if user_parsed_input.init() {
            return Ok(Self(base_path.to_owned()));
        }

        let name = user_parsed_input
            .name()
            .map_or_else(|| project_name_input.as_ref().to_owned(), String::from);

        let dir_name = user_parsed_input
            .force()
            .then(|| name.clone())
            .unwrap_or_else(|| {
                let renamed_project_name = name.to_kebab_case();
                if renamed_project_name != name {
                    warn!(
                        "{} `{}` {} `{}`{}",
                        style("Renaming project called").bold(),
                        style(name).bold().yellow(),
                        style("to").bold(),
                        style(&renamed_project_name).bold().green(),
                        style("...").bold()
                    );
                }
                renamed_project_name
            });

        let project_dir = base_path.join(dir_name);

        if project_dir.exists() {
            bail!(
                "{} {}",
                emoji::ERROR,
                style("Target directory already exists, aborting!")
                    .bold()
                    .red()
            );
        }

        Ok(Self(project_dir))
    }
}
