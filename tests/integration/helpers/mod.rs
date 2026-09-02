// Which helpers a build needs depends on the `git` cargo feature: most of
// the suite is gated on it, so a feature-off build legitimately leaves
// parts of the harness unused.
#![allow(dead_code)]

use crate::helpers::project::Project;
use crate::helpers::project_builder::tempdir;
use indoc::indoc;

pub mod arg_builder;
pub mod fake_proxy;
pub mod prelude;
pub mod project;
pub mod project_builder;

pub fn create_template(description: &str) -> Project {
    tempdir()
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
