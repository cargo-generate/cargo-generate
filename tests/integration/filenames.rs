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

#[test]
fn it_preserves_liquid_files_with_git() {
    assert_liquid_paths(Location::Git)
}

#[test]
fn it_preserves_liquid_files_with_path() {
    assert_liquid_paths(Location::Path)
}

#[derive(PartialEq)]
enum Location {
    Git,
    Path,
}

fn assert_liquid_paths(location: Location) {
    let mut project_builder = tempdir()
        .file("README.md", "remove me")
        .file("README.md.liquid", "keep me");

    if location == Location::Git {
        project_builder = project_builder.init_git();
    }

    let template = project_builder.build();

    let mut binary_command = binary();

    match location {
        Location::Git => {
            binary_command.arg_git(template.path());
        }
        Location::Path => {
            binary_command.arg_path(template.path());
        }
    }

    let dir = tempdir().build();

    binary_command
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.exists("foobar-project/README.md"),
        "project should contain foobar-project/README.md"
    );
    assert!(
        !dir.exists("foobar-project/README.md.liquid"),
        "project should not contain foobar-project/README.md.liquid"
    );

    assert_eq!(
        "keep me",
        dir.read("foobar-project/README.md"),
        "project should keep .liquid file contents"
    )
}
