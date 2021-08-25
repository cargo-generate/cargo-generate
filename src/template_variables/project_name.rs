use heck::{KebabCase, SnakeCase};

/// Stores user inputted name and provides convenience methods
/// for handling casing.
pub struct ProjectName {
    pub(crate) user_input: String,
}

impl ProjectName {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            user_input: name.into(),
        }
    }

    pub(crate) fn raw(&self) -> String {
        self.user_input.to_owned()
    }

    pub(crate) fn kebab_case(&self) -> String {
        self.user_input.to_kebab_case()
    }

    pub(crate) fn snake_case(&self) -> String {
        self.user_input.to_snake_case()
    }

    pub(crate) fn is_crate_name(&self) -> bool {
        self.user_input == self.kebab_case()
    }
}
