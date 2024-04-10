use console::style;
use std::fmt::Display;

use heck::{ToKebabCase, ToSnakeCase};
use log::warn;

use crate::user_parsed_input::UserParsedInput;

use super::ProjectNameInput;

#[derive(PartialOrd, PartialEq, Debug)]
pub struct ProjectName(String);

impl From<(&ProjectNameInput, &UserParsedInput)> for ProjectName {
    fn from(
        (project_name_input, user_parsed_input): (&ProjectNameInput, &UserParsedInput),
    ) -> Self {
        user_parsed_input
            .force()
            .then(|| ProjectName(project_name_input.as_ref().to_owned()))
            .unwrap_or_else(|| ProjectName::from(project_name_input.as_ref()))
    }
}

impl From<&str> for ProjectName {
    fn from(name: &str) -> Self {
        let dash_cased_project_name = name.to_kebab_case();
        let snake_case_project_name = name.to_snake_case();

        if snake_case_project_name == name {
            ProjectName(snake_case_project_name)
        } else {
            if dash_cased_project_name != name {
                warn!(
                    "{} `{}` {} `{}`{}",
                    style("Renaming project").bold(),
                    style(name).bold().yellow(),
                    style("to").bold(),
                    style(&dash_cased_project_name).bold().green(),
                    style("...").bold()
                );
            }
            ProjectName(dash_cased_project_name)
        }
    }
}

impl AsRef<str> for ProjectName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for ProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_parsed_input::UserParsedInputBuilder;

    #[test]
    fn test_snake_case_is_accepted() {
        let input = ProjectNameInput("lock_firmware".to_string());
        let args = UserParsedInputBuilder::for_testing().build();

        let project_name = ProjectName::try_from((&input, &args)).unwrap();
        assert_eq!(project_name, ProjectName("lock_firmware".into()));
    }

    #[test]
    fn test_dash_case_is_accepted() {
        let input = ProjectNameInput("lock-firmware".to_string());
        let args = UserParsedInputBuilder::for_testing().build();

        let project_name = ProjectName::try_from((&input, &args)).unwrap();
        assert_eq!(project_name, ProjectName("lock-firmware".into()));
    }

    #[test]
    fn test_converted_to_dash_case() {
        let input = ProjectNameInput("lockFirmware".to_string());
        let args = UserParsedInputBuilder::for_testing().build();

        let project_name = ProjectName::try_from((&input, &args)).unwrap();
        assert_eq!(project_name, ProjectName("lock-firmware".into()));
    }

    #[test]
    fn test_not_converted_to_dash_case_when_with_force() {
        let input = ProjectNameInput("lockFirmware".to_string());
        let args = UserParsedInputBuilder::for_testing().with_force().build();

        let project_name = ProjectName::try_from((&input, &args)).unwrap();
        assert_eq!(project_name, ProjectName("lockFirmware".into()));
    }
}
