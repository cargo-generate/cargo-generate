use heck::{KebabCase, SnakeCase};

/// Stores user inputted name and provides convenience methods
/// for handling casing.
pub struct ProjectName {
    pub user_input: String,
    pub kebab_case_name: String,
    pub snake_case_name: String,
}

impl ProjectName {
    pub fn new(name: &str) -> ProjectName {
        ProjectName {
            user_input: name.to_string(),
            kebab_case_name: name.to_kebab_case(),
            snake_case_name: name.to_snake_case(),
        }
    }

    pub fn kebab_case(&self) -> String {
        self.user_input.to_kebab_case()
    }

    pub fn snake_case(&self) -> String {
        self.user_input.to_snake_case()
    }
}
