use crate::helpers::project::Project;
use crate::helpers::project_builder::tmp_dir;
use indoc::indoc;

pub mod project;
pub mod project_builder;

pub fn create_template(description: &str) -> Project {
    tmp_dir()
        .file(
            "Cargo.toml",
            format!(
                indoc! {r#"
                    [package]
                    name = "{{{{project-name}}}}"
                    description = "{}"
                    version = "0.1.0"
                    "#},
                description
            )
            .as_str(),
        )
        .init_git()
        .build()
}
