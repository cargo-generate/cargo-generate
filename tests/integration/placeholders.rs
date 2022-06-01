use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use indoc::indoc;

#[test]
fn it_supports_boolean_placeholder_with_a_predefined_value() {
    let config_dir = tmp_dir()
        .file(
            "config",
            indoc! {r#"
                [values]
                smartmodule-params = true
                "#},
        )
        .build();

    let template_dir = tmp_dir()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [placeholders.smartmodule-params]
                type = "bool"
                prompt = "Want to use SmartModule parameters?"
                default = false
                "#},
        )
        .file(
            "random.toml",
            indoc! {r#"
                value = "{{smartmodule-params}}"
            "#},
        )
        .init_git()
        .build();

    let working_dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--config")
        .arg(config_dir.path().join("config"))
        .arg("--name")
        .arg("my-project")
        .arg("--git")
        .arg(template_dir.path())
        .current_dir(&working_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        working_dir
            .read("my-project/random.toml")
            .contains(r#"value = true"#),
        "given: {}",
        working_dir.read("my-project/random.toml")
    );
}
