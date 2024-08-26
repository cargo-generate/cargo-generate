use crate::helpers::prelude::*;

#[test]
fn should_work_with_a_gitconfig_that_has_a_ssh_instead_of_url() {
    let gitconfig_dir = tempdir()
        .file(
            ".gitconfig",
            indoc! { r#"
                [url "git@github.com:"]
                insteadOf = https://github.com/
            "# },
        )
        .build();

    let target = tempdir().build();

    binary()
        .arg_git("https://github.com/rustwasm/wasm-pack-template.git")
        .arg_gitconfig(gitconfig_dir.path().join(".gitconfig"))
        .arg_name("foobar-project")
        .current_dir(target.path())
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!").from_utf8().and(
                predicates::str::contains(
                    "gitconfig 'insteadOf' lead to this url: git@github.com:rustwasm/wasm-pack-template.git",
                )
                .from_utf8(),
            ),
        );

    let cargo_toml = target.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("foobar-project"));
}

#[test]
fn should_work_with_a_gitconfig_that_has_a_ssh_instead_of_url_with_ssh_prefix() {
    let gitconfig_dir = tempdir()
        .file(
            ".gitconfig",
            indoc! { r#"
                [url "ssh://git@github.com/"]
                insteadOf = https://github.com/
            "# },
        )
        .build();

    let target = tempdir().build();

    binary()
        .arg_git("https://github.com/rustwasm/wasm-pack-template.git")
        .arg_gitconfig(gitconfig_dir.path().join(".gitconfig"))
        .arg_name("foobar-project")
        .current_dir(target.path())
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!").from_utf8().and(
                predicates::str::contains(
                    "gitconfig 'insteadOf' lead to this url: ssh://git@github.com/rustwasm/wasm-pack-template.git",
                )
                .from_utf8(),
            ),
        );

    let cargo_toml = target.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("foobar-project"));
}
