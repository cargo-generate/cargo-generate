//! Handle `--git` and related flags

mod clone;
mod gitconfig;

mod gix_exploration;
mod init;
#[cfg(feature = "vendored-libgit2")]
mod repo_clone_builder_git2;
mod utils;

pub use clone::{clone_git_template_into_temp, clone_local_path_as_if_it_was_a_repo};
pub use init::init;

pub type BranchName = String;
