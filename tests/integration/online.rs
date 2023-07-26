//! Test that use real connection and working third part service.
//!
//! The test are ignored by default
//! You can run them with:
//! ```
//! cargo test -- --include-ignored
//! ````

use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;

#[test]
#[ignore]
fn git_flag_can_be_skipped_and_cargo_will_use_correct_implementation() {
    // with --git
    let dir = tmp_dir().build();
    binary()
        .arg_name("my-proj")
        .arg("--init")
        .arg("git://github.com/ashleygwilliams/wasm-pack-template")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}

#[test]
#[ignore]
fn plain_git_repo_works() {
    let possible_urls = vec![
        "git://github.com/ashleygwilliams/wasm-pack-template",
        "git://github.com/ashleygwilliams/wasm-pack-template.git",
        "https://github.com/ashleygwilliams/wasm-pack-template.git",
        "https://github.com/ashleygwilliams/wasm-pack-template",
        "http://github.com/ashleygwilliams/wasm-pack-template.git",
        "http://github.com/ashleygwilliams/wasm-pack-template",
    ];

    // with --git
    for remote in possible_urls {
        let dir = tmp_dir().build();
        binary()
            .arg_git(remote)
            .arg_name("my-proj")
            .arg("--init")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(predicates::str::contains("Done!").from_utf8());
    }
}

#[test]
#[ignore]
fn abbreviation_for_github_works() {
    let dir = tmp_dir().build();
    binary()

        .arg_name("my-proj")
        .arg("ashleygwilliams/wasm-pack-template")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").and(
            predicates::str::contains(
                "Favorite `ashleygwilliams/wasm-pack-template` not found in config, using it as a git repository: https://github.com/ashleygwilliams/wasm-pack-template.git"
            )).from_utf8());
}

#[cfg(test)]
#[cfg(unix)]
mod ssh_remote {
    use super::*;

    #[test]
    #[ignore]
    // for now only locally working
    fn it_should_support_a_public_repo() {
        let dir = tmp_dir().build();

        binary()
            .arg_git("git@github.com:ashleygwilliams/wasm-pack-template.git")
            .arg_name("foobar-project")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(predicates::str::contains("Done!").from_utf8());

        let cargo_toml = dir.read("foobar-project/Cargo.toml");
        assert!(cargo_toml.contains("foobar-project"));
    }

    #[test]
    #[ignore]
    // for now only locally working
    fn it_should_support_a_private_repo() {
        let dir = tmp_dir().build();

        binary()
            .arg_git("git@github.com:cargo-generate/wasm-pack-template.git")
            .arg_name("foobar-project")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(predicates::str::contains("Done!").from_utf8());

        let cargo_toml = dir.read("foobar-project/Cargo.toml");
        assert!(cargo_toml.contains("foobar-project"));
    }

    #[test]
    #[ignore]
    // for now only locally working
    fn it_should_support_a_custom_ssh_key() {
        let dir = tmp_dir().build();

        binary()
            .arg("-i")
            .arg("~/workspaces/rust/cargo-generate-org/.env/id_rsa_ci")
            .arg_git("git@github.com:cargo-generate/wasm-pack-template.git")
            .arg_name("foobar-project")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(
                predicates::str::contains("Using private key:")
                    .and(predicates::str::contains(
                        "cargo-generate-org/.env/id_rsa_ci",
                    ))
                    .from_utf8(),
            );

        let cargo_toml = dir.read("foobar-project/Cargo.toml");
        assert!(cargo_toml.contains("foobar-project"));
    }
}
