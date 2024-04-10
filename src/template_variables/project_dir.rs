use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::bail;
use console::style;

use crate::template_variables::ProjectName;
use crate::{emoji, user_parsed_input::UserParsedInput};

/// Stores user inputted name and provides convenience methods
/// for handling casing.
#[derive(PartialOrd, PartialEq, Debug)]
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

impl TryFrom<(&ProjectName, &UserParsedInput)> for ProjectDir {
    type Error = anyhow::Error;

    fn try_from(
        (project_name, user_parsed_input): (&ProjectName, &UserParsedInput),
    ) -> Result<Self, Self::Error> {
        let base_path = user_parsed_input.destination();

        if user_parsed_input.init() {
            return Ok(Self(base_path.to_owned()));
        }

        // let name = user_parsed_input
        //     .name()
        //     .map_or_else(|| project_name.as_ref().to_owned(), String::from);
        let project_dir = base_path.join(project_name.as_ref());

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_variables::ProjectNameInput;
    use crate::user_parsed_input::UserParsedInputBuilder;

    #[test]
    fn test_snake_case_is_accepted() {
        let input = ProjectName::from("lock_firmware");
        let args = UserParsedInputBuilder::for_testing().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert_eq!(project_dir, ProjectDir("/tmp/dest/lock_firmware".into()));
    }

    #[test]
    fn test_dash_case_is_accepted() {
        let input = ProjectName::from("lock-firmware");
        let args = UserParsedInputBuilder::for_testing().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert_eq!(project_dir, ProjectDir("/tmp/dest/lock-firmware".into()));
    }

    #[test]
    fn test_converted_to_dash_case() {
        let input = ProjectName::from("lockFirmware");
        let args = UserParsedInputBuilder::for_testing().build();

        let project_dir = ProjectDir::try_from((&input, &args)).unwrap();
        assert_eq!(project_dir, ProjectDir("/tmp/dest/lock-firmware".into()));
    }

    #[test]
    fn test_not_converted_to_dash_case_when_with_force() {
        let input = ProjectNameInput("lockFirmware".to_string());
        let args = UserParsedInputBuilder::for_testing().with_force().build();
        let project_name = ProjectName::try_from((&input, &args)).unwrap();

        let project_dir = ProjectDir::try_from((&project_name, &args)).unwrap();
        assert_eq!(project_dir, ProjectDir("/tmp/dest/lockFirmware".into()));
    }
}
