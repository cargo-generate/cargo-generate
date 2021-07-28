use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn it_substitutes_filename() {
    let template = tmp_dir()
        .file("main.rs", r#"extern crate {{crate_name}};"#)
        .file(
            "{{project-name}}.rs",
            r#"println!("Welcome in {{project-name}}");"#,
        )
        .init_git()
        .build();

    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/main.rs"), true);
    assert_eq!(dir.exists("foobar-project/foobar-project.rs"), true);
}
