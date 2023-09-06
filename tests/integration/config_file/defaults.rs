use crate::helpers::prelude::*;

#[test]
fn it_uses_ssh_identity_from_defaults_config() {
    let working_dir = tempdir().file("fake_ssh_identity", "fake stuff").build();
    let fake_ssh_id = working_dir.path().join("fake_ssh_identity");
    let config_dir = tempdir()
        .file(
            "cargo-generate.toml",
            format!(
                indoc! {r#"
                    [defaults]
                    ssh_identity = "{id}"
                "#},
                id = fake_ssh_id.display().to_string().escape_default()
            ),
        )
        .file("fake_ssh", "random foo") // FIXME: what is for?
        .build();
    let some_template = create_template("some-template");

    binary()
        .arg("--config")
        .arg(config_dir.path().join("cargo-generate.toml"))
        .arg_name("foo")
        .arg_git(some_template.path())
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!").from_utf8().and(
                predicates::str::contains("Using private key: ")
                    .from_utf8()
                    .and(predicates::str::contains("fake_ssh_identity").from_utf8()),
            ),
        );

    assert!(working_dir
        .read("foo/Cargo.toml")
        .contains(r#"name = "foo""#));
}
