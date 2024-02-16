use std::ops::Not;

use crate::helpers::prelude::*;

#[test]
fn it_can_conditionally_include_files() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! { r#"
                [template]
                exclude = ["excluded1", "excluded2"]

                [placeholders]
                foo = {type="bool", prompt="?"}

                [conditional.'!foo']
                ignore = ["included"]
            "# },
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("foo=false")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(!dir.exists("foobar-project/included"));
}

#[test]
fn it_can_conditionally_include_files2() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! { r#"
                [template]
                exclude = ["excluded1", "excluded2"]

                [placeholders]
                foo = {type="bool", prompt="?"}

                [conditional.'!foo']
                ignore = ["included"]
            "# },
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("foo=true")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/included")
        .contains("foobar-project"));

    assert!(dir
        .read("foobar-project/excluded1")
        .contains("{{should-not-process}}"));
    assert!(dir
        .read("foobar-project/excluded2")
        .contains("{{should-not-process}}"));
}

#[test]
fn it_can_ask_placeholders_in_multiple_levels() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! { r#"
                [placeholders]
                v1 = {type="bool", prompt="?"}

                [conditional.'v1'.placeholders]
                v2 = {type="bool", prompt="?"}

                [conditional.'v2']
                ignore = ["included"]
            "# },
        )
        .file("included", "{{project-name}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg("--silent")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("v1=true")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error:").from_utf8());
}

#[test]
fn it_supports_conditions_in_multiple_levels() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! { r#"
                [placeholders]
                v1 = {type="bool", prompt="?"}

                [conditional.'v1'.placeholders]
                v2 = {type="bool", prompt="?"}

                [conditional.'v2']
                ignore = ["included"]
            "# },
        )
        .file("included", "{{project-name}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg("--silent")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg("-d")
        .arg("v1=true")
        .arg("-d")
        .arg("v2=true")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.exists("foobar-project/included").not());
}

#[test]
fn it_supports_conditions_with_creepy_shit_inside() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [placeholders.platform]
                type = "string"
                prompt = "What platform are you targeting?"
                choices = ["web", "fullstack", "desktop", "liveview", "TUI"]
                default = "web"

                [conditional.'["liveview", "fullstack"].contains(platform)'.placeholders.backend]
                type = "string"
                prompt = "What backend framework are you using?"
                choices = ["Axum", "Warp", "Salvo"]
                default = "Axum"

                [conditional.'["web", "desktop", "fullstack"].contains(platform)'.placeholders.styling]
                type = "string"
                prompt = "How do you want to create CSS?"
                choices = ["Tailwind", "Vanilla"]
                default = "Vanilla"

                [conditional.'[Some("web").unwrap(), "desktop", "fullstack"].contains(platform)'.placeholders.styling2]
                type = "string"
                prompt = "How do you want to create CSS?"
                choices = ["Tailwind", "Vanilla"]
                default = "Vanilla"

                [conditional.'platform == "web" || backend == "Axum"'.placeholders.styling3]
                type = "string"
                prompt = "How do you want to create CSS?"
            "#},
        )
        .file("included", indoc! {r#"
            {{project-name}}
            platform = {{ platform }}
            backend = {{ backend }}
            styling = {{ styling }}
            styling2 = {{ styling2 }}
            styling3 = {{ styling3 }}
        "#})
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg("--silent")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .args(["-d", "platform=web"])
        .args(["-d", "backend=Axum"])
        .args(["-d", "styling=Tailwind"])
        .args(["-d", "styling2=Tailwind"])
        .args(["-d", "styling3=Tailwind"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/included")
        .contains("platform = web"));
    assert!(dir
        .read("foobar-project/included")
        .contains("backend = Axum"));
    assert!(dir
        .read("foobar-project/included")
        .contains("styling = Tailwind"));
    assert!(dir
        .read("foobar-project/included")
        .contains("styling2 = Tailwind"));
    assert!(dir
        .read("foobar-project/included")
        .contains("styling3 = Tailwind"));
}
