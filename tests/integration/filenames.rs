use crate::helpers::prelude::*;

#[test]
fn it_substitutes_filename() {
    let template = tempdir()
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

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.exists("foobar-project/main.rs"),
        "project should contain foobar-project/main.rs"
    );
    assert!(
        !dir.exists("foobar-project/{{project-name}}.rs"),
        "project should NOT contain foobar-project/{{project-name}}.rs"
    );
    assert!(
        dir.exists("foobar-project/foobar-project.rs"),
        "project should contain foobar-project/foobar-project.rs"
    );
    assert!(
        !dir.exists("foobar-project/src/{{project-name}}/lib.rs"),
        "project should NOT contain foobar-project/src/foobar-project/lib.rs.liquid"
    );
    assert!(
        dir.exists("foobar-project/src/foobar-project/lib.rs"),
        "project should contain foobar-project/src/foobar-project/lib.rs"
    );
}
