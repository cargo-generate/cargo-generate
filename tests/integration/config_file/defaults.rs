use predicates::prelude::*;

use crate::helpers::create_template;
use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use indoc::indoc;

#[test]
fn it_uses_ssh_identity_from_defaults_config() {
    let working_dir = tmp_dir().file("fake_ssh", "fake stuff").build();
    let fake_ssh_id = working_dir.path().join("fake_ssh_identity");
    let config_dir = tmp_dir()
        .file(
            "cargo-generate.toml",
            &format!(
                indoc! {r#"
                    [defaults]
                    ssh_identity = "{id}"
                "#},
                id = fake_ssh_id.display().to_string().escape_default()
            ),
        )
        .file("fake_ssh_identity", "random foo")
        .build();
    let some_template = create_template("some-template");

    binary()
        .arg("generate")
        .arg("--config")
        .arg(config_dir.path().join("cargo-generate.toml"))
        .arg("--name")
        .arg("foo")
        .arg("--git")
        .arg(some_template.path())
        .current_dir(&working_dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!").from_utf8().and(
                predicates::str::contains("Using ssh-identity from application config: ")
                    .from_utf8()
                    .and(predicates::str::contains("fake_ssh_identity").from_utf8()),
            ),
        );

    assert!(working_dir
        .read("foo/Cargo.toml")
        .contains(r#"name = "foo""#));
}
