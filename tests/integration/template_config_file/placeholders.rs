use crate::helpers::prelude::*;

#[test]
fn it_prompts_for_placeholders_in_the_config_file_defined_order() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                [placeholders.mcu]
                type = "string"
                prompt = "Which MCU to target?"
                choices = ["esp32", "esp32c2", "esp32c3", "esp32c6", "esp32s2", "esp32s3"]
                default = "esp32"

                [placeholders.defaults]
                type = "bool"
                prompt = "Use template default values?"
                default = true
            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .args(["--define", "defaults=true"])
        .args(["--define", "mcu=esp32"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::is_match(r"defaults.*\n.*mcu").unwrap());
}

#[test]
fn it_substitutes_multi_selections() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                [placeholders.formats]
                type = "array"
                prompt = "Which MCU to target?"
                choices = ["esp32", "esp32c2", "esp32c3", "esp32c6", "esp32s2", "esp32s3"]
                default = ["esp32"]

            "#},
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .args(["--define", "formats=esp32,esp32c3"])
        .current_dir(dir.path())
        .assert()
        .success();
}
