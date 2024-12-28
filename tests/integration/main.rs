mod helpers;

// test modules go here
mod basics;
mod conditionals;
mod config_file;
mod filenames;
mod git;
mod git_instead_of;
#[cfg(e2e_tests_with_ssh_key)]
mod git_over_ssh;
mod hooks_and_rhai;
mod public_api;
mod template_config_file;
mod template_filters;
mod workspace_member;
