use heck::{KebabCase, SnakeCase};

/// Stores user inputted name and provides convenience methods
/// for handling casing.
pub struct ProjectName {
    user_input: String,
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
}
