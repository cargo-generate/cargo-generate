use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use indoc::indoc;

#[test]
fn it_prompts_for_placeholders_in_the_config_file_defined_order() {
    let template = tmp_dir()
        .default_manifest()
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

    let dir = tmp_dir().build();

    binary()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .arg("--branch")
        .arg("main")
        .args(["--define", "mcu=esp32"])
        .args(["--define", "defaults=true"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains(indoc! {r#"
                🔧   mcu: "esp32" (variable provided via CLI)
                🔧   defaults: "true" (variable provided via CLI)
                "#})
            .from_utf8(),
        );
}
