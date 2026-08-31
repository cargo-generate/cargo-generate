use crate::helpers::prelude::*;

// A cargo `target/` directory left inside a template must not be walked,
// templated, or copied into the generated project — otherwise generation
// is dramatically slowed by incremental build artifacts.
//
// Detection is strict: only directories carrying a `CACHEDIR.TAG` marker
// (the convention cargo writes into every build directory) are skipped, so
// templates that legitimately ship a `target/` directory of their own are
// untouched.
//
// Regression test for https://github.com/cargo-generate/cargo-generate/issues/1600
#[test]
fn issue_1600_target_directory_with_cachedir_tag_is_excluded() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "target/CACHEDIR.TAG",
            "Signature: 8a477f597d28d172789f06886806bc55",
        )
        .file("target/debug/deps/libfoo.rlib", "not a real artifact")
        .file("target/debug/build/foo/output", "cargo build output marker")
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success();

    let generated_target = dir.path().join("foobar-project").join("target");
    assert!(
        !generated_target.exists(),
        "target/ directory (marked with CACHEDIR.TAG) was copied to the generated \
         project, indicating cargo-generate is still walking rust build artifacts \
         (issue #1600)"
    );
}

// Counterpart: a `target/` directory that does NOT carry a `CACHEDIR.TAG`
// marker is a plain user directory and must be preserved. This guards the
// strict-detection contract against accidental over-matching.
#[test]
fn a_plain_target_directory_without_cachedir_tag_is_preserved() {
    let template = tempdir()
        .with_default_manifest()
        .file("target/notes.txt", "user-authored content")
        .build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success();

    let generated_notes = dir
        .path()
        .join("foobar-project")
        .join("target")
        .join("notes.txt");
    assert!(
        generated_notes.exists(),
        "target/ without CACHEDIR.TAG must be treated as a normal directory and copied"
    );
}
