//! Test that use real connection, upstream github repo and an specific ssh-key
//!
//! This test cases need ssh-agend on windows to work.
//! See https://github.com/cargo-generate/cargo-generate/discussions/653
//!
//! For reasons not yet known, there are issues on windows:
//!     - The ssh-agent does not work as expected
//!     - Additionally, the ssh-key via `--identity` is not working as expected
use crate::helpers::prelude::*;

#[test]
fn it_should_fail_if_a_identity_file_does_not_exist() {
    let dir = tempdir().build();

    binary()
        .arg_identity("id_foobarbak")
        .arg_git("git@github.com:rustwasm/wasm-pack-template.git")
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error: path does not exist: id_foobarbak").from_utf8());
}

#[cfg(unix)]
#[test]
#[ignore]
// for now only locally working
fn it_should_support_a_public_repo() {
    let dir = tempdir().build();

    binary()
        .arg_git("git@github.com:rustwasm/wasm-pack-template.git")
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("foobar-project"));
}

#[cfg(unix)]
#[test]
#[ignore]
// for now only locally working
fn it_should_retrieve_the_private_key_from_ssh_agent() {
    let ssh_urls_for_repos = [
        "git@github.com:cargo-generate/wasm-pack-template.git",
        "ssh://git@github.com/cargo-generate/wasm-pack-template.git",
    ];

    for ssh_repo_url in ssh_urls_for_repos {
        let dir = tempdir().build();

        binary()
            .arg_git(ssh_repo_url)
            .arg_name("foobar-project")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(predicates::str::contains("Done!").from_utf8());

        let cargo_toml = dir.read("foobar-project/Cargo.toml");
        assert!(cargo_toml.contains("foobar-project"));
    }
}

// run this as:
// ```sh
// RUST_LOG=debug CARGO_GENERATE_E2E_SSH_PRIVATE_KEY=~/.ssh/id_cargo-generate-e2e-test-key cargo test
// ```
#[cfg(unix)]
#[test]
fn it_should_use_a_ssh_key_provided_by_identity_argument() {
    let Ok(private_key) = env::var("CARGO_GENERATE_E2E_SSH_PRIVATE_KEY") else {
        panic!("Skipping test because CARGO_GENERATE_E2E_SSH_PRIVATE_KEY is not set");
    };

    let ssh_urls_for_repos = [
        "git@github.com:cargo-generate/wasm-pack-template.git",
        "ssh://git@github.com/cargo-generate/wasm-pack-template.git",
    ];

    for ssh_repo_url in ssh_urls_for_repos {
        let dir = tempdir().build();

        binary()
            .arg_identity(private_key.as_str())
            .arg_git(ssh_repo_url)
            .arg_name("foobar-project")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(
                predicates::str::contains("Using private key:")
                    .and(predicates::str::contains(private_key.as_str()))
                    .from_utf8(),
            );

        let cargo_toml = dir.read("foobar-project/Cargo.toml");
        assert!(cargo_toml.contains("foobar-project"));
    }
}
