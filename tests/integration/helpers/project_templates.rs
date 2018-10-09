use helpers::project_builder::{dir, ProjectBuilder};

pub fn cargo_template_with_copyright() -> ProjectBuilder {
    dir("template").file(
        "Cargo.toml",
        r#"[package]
name = "{{project-name}}"
description = "A wonderful project Copyright {{ "2018-10-04 18:18:45 +0200" | date: "%Y" }}"
version = "0.1.0"
"#,
    )
}

pub fn default_cargo_template() -> ProjectBuilder {
    dir("template").file(
        "Cargo.toml",
        r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
    )
}
