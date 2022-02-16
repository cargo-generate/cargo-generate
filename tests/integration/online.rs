//! Thest that use real connection and working third part service.
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
fn schema_can_be_skiped_with_git() { 
    let dir = tmp_dir().build();
    binary()
        .arg("generate")
        .arg("--git")
        .arg("github.com/ashleygwilliams/wasm-pack-template")
        .arg("--name")
        .arg("my-proj")
        .arg("--init")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}

#[test]
#[ignore]
fn schema_can_be_skiped_with_favorite() { 
    let dir = tmp_dir().build();
    binary()
        .arg("generate")
        .arg("--name")
        .arg("my-proj")
        .arg("--init")
        .arg("github.com/ashleygwilliams/wasm-pack-template")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}

#[test]
#[ignore]
fn abbreviation_for_github_works() { 
    let dir = tmp_dir().build();
    binary()
        .arg("generate")
        .arg("--name")
        .arg("my-proj")
        .arg("ashleygwilliams/wasm-pack-template")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
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
            .arg("generate")
            .arg("--git")
            .arg("git@github.com:ashleygwilliams/wasm-pack-template.git")
            .arg("--name")
            .arg("foobar-project")
            .current_dir(&dir.path())
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
            .arg("generate")
            .arg("--git")
            .arg("git@github.com:cargo-generate/wasm-pack-template.git")
            .arg("--name")
            .arg("foobar-project")
            .current_dir(&dir.path())
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
            .arg("generate")
            .arg("-i")
            .arg("~/workspaces/rust/cargo-generate-org/.env/id_rsa_ci")
            .arg("--git")
            .arg("git@github.com:cargo-generate/wasm-pack-template.git")
            .arg("--name")
            .arg("foobar-project")
            .current_dir(&dir.path())
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
