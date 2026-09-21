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

/// `cargo-fmt` is an identifier, not a filename, so a template file that
/// happens to carry that name is templated and copied like any other.
#[test]
fn a_template_file_named_cargo_fmt_is_still_copied() {
    let template = tempdir()
        .with_default_manifest()
        .file("src/main.rs", UNFORMATTED_MAIN)
        .file("cargo-fmt", "I belong to {{project-name}}\n")
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [hooks]
                post = ["cargo-fmt"]
            "#},
        )
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success();

    // the file survived as content...
    assert_eq!(
        dir.read("fmt-project/cargo-fmt"),
        "I belong to fmt-project\n"
    );
    // ...and the identifier still did its job
    assert_eq!(dir.read("fmt-project/src/main.rs"), FORMATTED_MAIN);
}

/// The identifier only shadows the bare name. A real hook script keeps working,
/// including one named `cargo-fmt.rhai`.
#[test]
fn a_hook_script_named_cargo_fmt_rhai_still_runs() {
    let template = tempdir()
        .with_default_manifest()
        .file("src/main.rs", UNFORMATTED_MAIN)
        .file("cargo-fmt.rhai", r#"file::rename("RENAME-ME", "renamed");"#)
        .file("RENAME-ME", "content")
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [hooks]
                post = ["cargo-fmt.rhai"]
            "#},
        )
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success();

    // the script ran...
    assert!(dir.exists("fmt-project/renamed"));
    // ...and was removed from the output like any hook file
    assert!(!dir.exists("fmt-project/cargo-fmt.rhai"));
    // it is not the identifier, so it did not request formatting on its own —
    // the default did
    assert_eq!(dir.read("fmt-project/src/main.rs"), FORMATTED_MAIN);
}

/// Regression test: `--init` expands into an existing project, and the code
/// already living there is not ours to reformat.
#[test]
fn it_only_formats_the_files_it_generated_under_init() {
    let template = tempdir()
        .file(
            "src/generated.rs",
            "pub fn generated(){println!(\"hi\");}\n",
        )
        .init_git()
        .build();

    // an existing crate with deliberately unformatted code of its own
    let existing = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "existing"
                version = "0.1.0"
                edition = "2021"
            "#},
        )
        .file("src/lib.rs", "pub fn theirs(){println!(\"theirs\");}\n")
        .build();

    binary()
        .arg_git(template.path())
        .arg_name("whatever")
        .flag_init()
        .current_dir(existing.path())
        .assert()
        .success();

    // what the template generated was formatted...
    assert_eq!(
        existing.read("src/generated.rs"),
        indoc! {r#"
            pub fn generated() {
                println!("hi");
            }
        "#}
    );
    // ...and the user's own file was left exactly as it was
    assert_eq!(
        existing.read("src/lib.rs"),
        "pub fn theirs(){println!(\"theirs\");}\n"
    );
}

/// The guarantee that makes formatting-by-default acceptable: a generated
/// project that rustfmt cannot parse is still a successful generation.
#[test]
fn a_rustfmt_failure_does_not_fail_generation() {
    let template = tempdir()
        .with_default_manifest()
        .file("src/main.rs", "fn main( { this is not rust at all ;;;\n")
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Could not format the generated files")
                .and(predicates::str::contains("Done!")),
        );

    // the unformattable file is still there, verbatim
    assert_eq!(
        dir.read("fmt-project/src/main.rs"),
        "fn main( { this is not rust at all ;;;\n"
    );
}

/// The whole reason formatting is file-scoped rather than `cargo fmt --all`:
/// the generated project is added to an enclosing workspace as a member, and
/// that workspace's other crates must come through untouched.
#[test]
fn it_does_not_reformat_an_enclosing_workspace() {
    let template = template_with(None);

    let workspace = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                resolver = "2"
                members = ["existing"]
            "#},
        )
        .file(
            "existing/Cargo.toml",
            indoc! {r#"
                [package]
                name = "existing"
                version = "0.1.0"
                edition = "2021"
            "#},
        )
        .file(
            "existing/src/lib.rs",
            "pub fn sibling(){println!(\"s\");}\n",
        )
        .build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(workspace.path())
        .assert()
        .success();

    // the generated member was formatted...
    assert_eq!(workspace.read("fmt-project/src/main.rs"), FORMATTED_MAIN);
    // ...the pre-existing sibling was not
    assert_eq!(
        workspace.read("existing/src/lib.rs"),
        "pub fn sibling(){println!(\"s\");}\n"
    );
}

/// A misplaced identifier warns once per phase, not once per internal read of
/// the hook lists.
#[test]
fn a_misplaced_identifier_warns_exactly_once_per_phase() {
    let template = tempdir()
        .with_default_manifest()
        .file("src/main.rs", UNFORMATTED_MAIN)
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [hooks]
                init = ["cargo-fmt"]
                pre = ["cargo-fmt"]
            "#},
        )
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("ignoring it in `init`")
                .count(1)
                .and(predicates::str::contains("ignoring it in `pre`").count(1)),
        );

    // ignored there, but the default still formatted the output
    assert_eq!(dir.read("fmt-project/src/main.rs"), FORMATTED_MAIN);
}

/// Doc claim: "Each file is formatted against the edition of the cargo package
/// it lands in, so a generated workspace whose members differ in edition is
/// handled correctly."
///
/// The two files are mutually exclusive across editions — `async` is an
/// identifier in 2015 and a keyword after it — so both coming out formatted is
/// only possible if each was formatted against its own member's edition.
#[test]
fn each_file_is_formatted_against_its_own_package_edition() {
    let template = tempdir()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [workspace]
                resolver = "2"
                members = ["old", "new"]
            "#},
        )
        .file(
            "old/Cargo.toml",
            indoc! {r#"
                [package]
                name = "old"
                version = "0.1.0"
                edition = "2015"
            "#},
        )
        .file(
            "old/src/lib.rs",
            "pub fn f(){let async=1;println!(\"{}\",async);}\n",
        )
        .file(
            "new/Cargo.toml",
            indoc! {r#"
                [package]
                name = "new"
                version = "0.1.0"
                edition = "2021"
            "#},
        )
        .file("new/src/lib.rs", "pub async fn g(){println!(\"g\");}\n")
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("ws-project")
        .current_dir(dir.path())
        .assert()
        .success()
        // neither member may have tripped over the other's edition
        .stdout(predicates::str::contains("Could not format").not());

    assert_eq!(
        dir.read("ws-project/old/src/lib.rs"),
        indoc! {r#"
            pub fn f() {
                let async = 1;
                println!("{}", async);
            }
        "#}
    );
    assert_eq!(
        dir.read("ws-project/new/src/lib.rs"),
        indoc! {r#"
            pub async fn g() {
                println!("g");
            }
        "#}
    );
}

/// Doc claim: in `init` or `pre` the identifier "is ignored with a warning".
///
/// The warning alone does not prove it is ignored — with formatting on by
/// default the output looks the same either way. Pairing it with
/// `fmt = false` is what makes "ignored" observable.
#[test]
fn a_misplaced_identifier_does_not_request_formatting() {
    let template = template_with(Some(indoc! {r#"
        [template]
        fmt = false

        [hooks]
        init = ["cargo-fmt"]
    "#}));
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("ignoring it in `init`"));

    assert_eq!(dir.read("fmt-project/src/main.rs"), EXPANDED_MAIN);
}

/// Doc claim: "If rustfmt is not on `PATH` \[...\] `cargo-generate` warns and
/// carries on with a generated project that is simply unformatted."
#[test]
fn a_missing_rustfmt_does_not_fail_generation() {
    let template = template_with(None);
    let dir = tempdir().build();

    let empty = tempdir().build();

    binary()
        // `--path` rather than `--git`: emptying PATH below also hides `git`,
        // which the git transport shells out to.
        .arg_path(template.path())
        .arg_name("fmt-project")
        .current_dir(dir.path())
        // an empty PATH is the cheapest stand-in for "the rustfmt component
        // was never installed"
        .env("PATH", empty.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Could not format the generated files")
                .and(predicates::str::contains("Done!")),
        );

    assert_eq!(dir.read("fmt-project/src/main.rs"), EXPANDED_MAIN);
}
