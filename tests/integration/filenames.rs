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
        .file(
            "src/{{project-name}}/lib.rs.liquid",
            r#"println!("Welcome in {{project-name}}-lib");"#,
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

    assert!(
        dir.exists("foobar-project/main.rs"),
        "project should contain foobar-project/main.rs"
    );
    assert!(
        dir.exists("foobar-project/foobar-project.rs"),
        "project should contain foobar-project/foobar-project.rs"
    );
    assert!(
        dir.exists("foobar-project/src/foobar-project/lib.rs"),
        "project should contain foobar-project/src/foobar-project/lib.rs"
    );
    assert!(
        !dir.exists("foobar-project/src/{{project-name}}/lib.rs"),
        "project should not contain foobar-project/src/foobar-project/lib.rs.liquid"
    );
}
