use crate::helpers::prelude::*;

/// Deliberately mis-formatted, so that "was it formatted?" is unambiguous.
const UNFORMATTED_MAIN: &str = "fn main(){println!(\"{{project-name}}\");}\n";

/// The same file once rustfmt has had it, for the generated `fmt-project`.
const FORMATTED_MAIN: &str = indoc! {r#"
    fn main() {
        println!("fmt-project");
    }
"#};

/// What the expansion produces when nothing formats it.
const EXPANDED_MAIN: &str = "fn main(){println!(\"fmt-project\");}\n";

fn template_with(config: Option<&str>) -> Project {
    let template = tempdir()
        .with_default_manifest()
        .file("src/main.rs", UNFORMATTED_MAIN);

    match config {
        Some(config) => template.file("cargo-generate.toml", config),
        None => template,
    }
    .init_git()
    .build()
}

#[test]
fn it_formats_the_generated_project_by_default() {
    let template = template_with(None);
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(dir.read("fmt-project/src/main.rs"), FORMATTED_MAIN);
}

#[test]
fn it_does_not_format_when_no_fmt_is_given() {
    let template = template_with(None);
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .arg("--no-fmt")
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(dir.read("fmt-project/src/main.rs"), EXPANDED_MAIN);
}

#[test]
fn it_does_not_format_when_the_template_opts_out() {
    let template = template_with(Some(indoc! {r#"
        [template]
        fmt = false
    "#}));
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(dir.read("fmt-project/src/main.rs"), EXPANDED_MAIN);
}

/// The intended authoring pattern from
/// <https://github.com/cargo-generate/cargo-generate/issues/1776>: opt out of
/// the automatic behaviour, then hook formatting back in explicitly.
#[test]
fn it_formats_when_the_named_post_hook_is_listed() {
    let template = template_with(Some(indoc! {r#"
        [template]
        fmt = false

        [hooks]
        post = ["cargo-fmt"]
    "#}));
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(dir.read("fmt-project/src/main.rs"), FORMATTED_MAIN);
    // the identifier is not a script file, so nothing named after it is emitted
    assert!(!dir.exists("fmt-project/cargo-fmt"));
}

#[test]
fn no_fmt_beats_the_named_post_hook() {
    let template = template_with(Some(indoc! {r#"
        [hooks]
        post = ["cargo-fmt"]
    "#}));
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .arg("--no-fmt")
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(dir.read("fmt-project/src/main.rs"), EXPANDED_MAIN);
}

/// A template that is not a cargo project must generate cleanly — the
/// `Cargo.toml` gate is what makes the default safe for them.
#[test]
fn it_is_a_silent_no_op_without_a_manifest() {
    let template = tempdir()
        .file("README.md", "# {{project-name}}\n")
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Formatting the generated project").not());

    assert_eq!(dir.read("fmt-project/README.md"), "# fmt-project\n");
}
