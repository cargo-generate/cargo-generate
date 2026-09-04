// Which of these a given test module needs depends on the `git` cargo
// feature — most of the suite is gated on it, so re-exports that every
// build uses can still look unused in a feature-off build.
#![cfg_attr(not(feature = "git"), allow(unused_imports))]

pub use crate::helpers::arg_builder::*;
pub use crate::helpers::create_template;
pub use crate::helpers::fake_proxy::FakeProxy;
pub use crate::helpers::project::Project;
pub use crate::helpers::project_builder::tempdir;

pub use assert_cmd::prelude::*;
pub use indoc::indoc;
pub use predicates::prelude::*;
pub use std::env;
pub use std::fs;
pub use std::ops::Not;
pub use std::path::PathBuf;
pub use std::process::Command;
