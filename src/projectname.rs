use heck::{KebabCase, SnakeCase};

pub struct ProjectName {
    pub kebab_case: String,
    pub snake_case: String,
}

impl ProjectName {
    pub fn new(name: &str) -> ProjectName {
        ProjectName {
            kebab_case: name.to_kebab_case(),
            snake_case: name.to_snake_case(),
        }
    }
}
