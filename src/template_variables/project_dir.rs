use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::bail;
use console::style;

use crate::template_variables::project_name::sanitize_project_name;
use crate::{emoji, user_parsed_input::UserParsedInput};
use log::warn;

use super::project_name_input::ProjectNameInput;

/// Stores user inputted name and provides convenience methods
/// for handling casing.
#[derive(Debug, PartialEq)]
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

        let dir_name = if user_parsed_input
            .force() { name.clone() } else { {
                let renamed_project_name = sanitize_project_name(name.as_str());
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
            } };

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

impl ProjectDir {
    pub fn create(&self) -> anyhow::Result<()> {
        let path = self.0.as_path();
        if path.exists() {
            bail!(
                "{} {}",
                emoji::ERROR,
                style("Target directory already exists, aborting!")
                    .bold()
                    .red()
            );
        }

        std::fs::create_dir(&self.0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_variables::ProjectNameInput;
    use crate::user_parsed_input::UserParsedInputBuilder;

    #[test]
    fn test_snake_case_is_accepted() {
        let input = ProjectNameInput("lock_firmware".to_string());
        let args = UserParsedInputBuilder::for_testing().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert!(project_dir.0.as_path().ends_with("lock_firmware"));
    }

    #[test]
    fn test_dash_case_is_accepted() {
        let input = ProjectNameInput("lock-firmware".to_string());
        let args = UserParsedInputBuilder::for_testing().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert!(project_dir.0.as_path().ends_with("lock-firmware"));
    }

    #[test]
    fn test_converted_to_dash_case() {
        let input = ProjectNameInput("lockFirmware".to_string());
        let args = UserParsedInputBuilder::for_testing().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert!(project_dir.0.as_path().ends_with("lock-firmware"));
    }

    #[test]
    fn test_not_converted_to_dash_case_when_with_force() {
        let input = ProjectNameInput("lockFirmware".to_string());
        let args = UserParsedInputBuilder::for_testing().with_force().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert!(project_dir.0.as_path().ends_with("lockFirmware"));
    }
}
