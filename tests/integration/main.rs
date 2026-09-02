mod helpers;

// test modules go here
mod basics;
#[cfg(feature = "git")]
mod conditionals;
mod config_file;
mod filenames;
#[cfg(feature = "git")]
mod git;
#[cfg(feature = "git")]
mod git_instead_of;
#[cfg(all(e2e_tests_with_ssh_key, feature = "git"))]
mod git_over_ssh;
mod hooks_and_rhai;
#[cfg(feature = "git")]
mod proxy;
mod public_api;
mod target_folder;
#[cfg(feature = "git")]
mod template_config_file;
#[cfg(feature = "git")]
mod template_filters;
mod workspace_member;
