use heck::{KebabCase, SnakeCase};

/// Stores user inputted name and provides convenience methods
/// for handling casing.
pub struct ProjectName {
    pub user_input: String,
}

impl ProjectName {
    pub fn new(name: &str) -> ProjectName {
        ProjectName {
            user_input: name.to_string(),
        }
    }

    pub fn kebab_case(&self) -> String {
        self.user_input.to_kebab_case()
    }

    pub fn snake_case(&self) -> String {
        self.user_input.to_snake_case()
    }

    pub fn is_crate_name(&self) -> bool {
        self.user_input == self.kebab_case()
    }
}
