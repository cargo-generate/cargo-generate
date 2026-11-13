use crate::helpers::prelude::*;
use indoc::indoc;

#[test]
fn final_hooks_see_expanded_files_and_variables_without_shipping_scripts() {
    let template = tempdir()
        .file("cargo-generate.toml", indoc! {r#"
            [hooks]
            pre = ["hooks/pre.rhai"]
            post = ["hooks/post.rhai"]
            final = ["hooks/../hooks/final.rhai"]
        "#})
        .file("hooks/pre.rhai", r#"variable::set("extra", "from-pre");"#)
        .file("hooks/post.rhai", r#"file::write("post-marker", "post");"#)
        .file("hooks/final.rhai", indoc! {r#"
            if !file::exists("rendered.txt") || !file::exists("post-marker") {
                throw "final must follow expansion and post";
            }
            if file::exists("hooks/final.rhai") || file::exists("hooks/pre.rhai") {
                throw "hook scripts must not be copied to the destination";
            }
            file::write("final-marker", variable::get("extra") + ":" + variable::get("project-name"));
        "#})
        .file("rendered.txt", "{{extra}}/{{project-name}}")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .success();
    assert_eq!(
        output.read("final-project/rendered.txt"),
        "from-pre/final-project"
    );
    assert_eq!(
        output.read("final-project/final-marker"),
        "from-pre:final-project"
    );
    for script in ["pre.rhai", "post.rhai", "final.rhai"] {
        assert!(!output.exists(&format!("final-project/hooks/{script}")));
    }
    assert!(template.exists("hooks/final.rhai"));
    assert!(!template.exists("final-marker"));
}

#[test]
fn final_hooks_operate_in_an_existing_init_destination() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file(
            "final.rhai",
            indoc! {r#"
            if !file::exists("existing.txt") { throw "wrong working directory"; }
            file::rename("existing.txt", "renamed.txt");
            file::write("final-marker", variable::get("project-name"));
        "#},
        )
        .file("generated.txt", "{{project-name}}")
        .build();
    let output = tempdir()
        .file("existing.txt", "keep this")
        .file("final.rhai", "throw \"destination script must not run\";")
        .build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .flag_init()
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .success();
    assert_eq!(output.read("renamed.txt"), "keep this");
    assert_eq!(output.read("final-marker"), "final-project");
    assert_eq!(output.read("generated.txt"), "final-project");
    assert!(!output.exists("existing.txt"));
    assert_eq!(
        output.read("final.rhai"),
        "throw \"destination script must not run\";"
    );
}

#[test]
fn final_hook_failure_reports_an_error_and_preserves_the_generated_project() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file("final.rhai", r#"throw "final failed deliberately";"#)
        .file("generated.txt", "already copied")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("final failed deliberately"))
        .stderr(predicates::str::contains("Done!").not());
    assert_eq!(output.read("final-project/generated.txt"), "already copied");
    assert!(!output.exists("final-project/final.rhai"));
}

#[test]
fn final_hook_commands_still_require_permission_in_silent_mode() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file(
            "final.rhai",
            r#"file::write("command-result", system::command("echo", ["permitted"]));"#,
        )
        .file("generated.txt", "generated")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Cannot prompt for system command confirmation",
        ));
    assert!(!output.exists("final-project/command-result"));
}

#[test]
fn final_hook_commands_run_when_explicitly_allowed() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file(
            "final.rhai",
            r#"file::write("command-result", system::command("echo", ["permitted"]));"#,
        )
        .file("generated.txt", "generated")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .flag_allow_commands()
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .success();
    assert_eq!(output.read("final-project/command-result"), "permitted");
}

#[test]
fn final_hooks_can_import_a_script_used_in_an_earlier_phase() {
    let template = tempdir()
        .file(
            "cargo-generate.toml",
            "[hooks]\npre = [\"hooks/helper.rhai\"]\nfinal = [\"hooks/final.rhai\"]",
        )
        .file(
            "hooks/helper.rhai",
            r#"fn message() { "temporary module" }"#,
        )
        .file(
            "hooks/final.rhai",
            r#"import "hooks/helper" as helper; file::write("final-marker", helper::message());"#,
        )
        .file("generated.txt", "generated")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .success();
    assert_eq!(
        output.read("final-project/final-marker"),
        "temporary module"
    );
    assert!(!output.exists("final-project/hooks/helper.rhai"));
    assert!(!output.exists("final-project/hooks/final.rhai"));
}

#[test]
fn final_hook_file_operations_remain_within_the_destination() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file(
            "final.rhai",
            r#"file::write("../outside.txt", "must not escape");"#,
        )
        .file("generated.txt", "generated")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .current_dir(output.path())
        .args(["--silent", "--vcs", "none"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Path must be inside template dir",
        ));
    assert!(!output.exists("outside.txt"));
}

#[cfg(feature = "git")]
#[test]
fn final_hooks_follow_git_initialization_and_commands_use_the_destination() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file(
            "final.rhai",
            indoc! {r#"
            if !file::exists(".git") { throw "git must be initialized first"; }
            file::write("git-prefix", system::command("git", ["rev-parse", "--show-prefix"]));
        "#},
        )
        .file("generated.txt", "generated")
        .build();
    let output = tempdir().build();
    binary()
        .arg_path(template.path())
        .arg_name("final-project")
        .flag_allow_commands()
        .current_dir(output.path())
        .arg("--silent")
        .assert()
        .success();
    assert!(output.exists("final-project/.git"));
    assert_eq!(output.read("final-project/git-prefix"), "");
}

#[cfg(feature = "git")]
#[test]
fn final_hooks_run_before_the_test_command_in_the_temporary_project() {
    let template = tempdir()
        .file("cargo-generate.toml", "[hooks]\nfinal = [\"final.rhai\"]")
        .file("final.rhai", r#"file::write("final-marker", "ready");"#)
        .file("expected.txt", "ready")
        .build();
    binary()
        .arg_name("final-project")
        .current_dir(template.path())
        .args(["--silent", "--vcs", "none", "--test"])
        .env(
            "CARGO_GENERATE_TEST_CMD",
            "git diff --no-index --exit-code expected.txt final-marker",
        )
        .assert()
        .success();
    assert!(!template.exists("final-project/final-marker"));
    assert!(!template.exists("final-marker"));
}
