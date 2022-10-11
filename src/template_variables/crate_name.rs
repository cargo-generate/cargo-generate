use std::fmt::Display;

use heck::ToSnakeCase;

use super::ProjectNameInput;

#[derive(Debug)]
pub struct CrateName(String);

impl From<&ProjectNameInput> for CrateName {
    fn from(project_name_input: &ProjectNameInput) -> Self {
        let crate_name = project_name_input.as_ref().to_snake_case();
        Self(crate_name)
    }
}

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for CrateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
