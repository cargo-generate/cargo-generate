use predicates::prelude::*;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;

#[test]
fn read_configuration_from_gitconfig() {
    let template = tmp_dir()
        .file(
            ".gitconfig",
            r#"
[url "https://github.com/"]
    insteadOf = ssh://git@github.com:
"#,
        )
        .build();

    let git_config = template.path().join(".gitconfig");
    let remote = "ssh://git@github.com:ashleygwilliams/wasm-pack-template";

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--name")
        .arg("foobar-project")
        .arg(remote)
        .env("GIT_CONFIG_GLOBAL", git_config)
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
}
