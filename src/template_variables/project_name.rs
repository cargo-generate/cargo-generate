use std::fmt::Display;

use heck::ToKebabCase;

use crate::user_parsed_input::UserParsedInput;

use super::ProjectNameInput;

#[derive(Debug)]
pub struct ProjectName(String);

impl From<(&ProjectNameInput, &UserParsedInput)> for ProjectName {
    fn from(
        (project_name_input, user_parsed_input): (&ProjectNameInput, &UserParsedInput),
    ) -> Self {
        Self(
            user_parsed_input
                .force()
                .then(|| project_name_input.as_ref().to_owned())
                .unwrap_or_else(|| project_name_input.as_ref().to_kebab_case()),
        )
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
