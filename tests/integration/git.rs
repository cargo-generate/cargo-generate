use bstr::ByteSlice;
use gix_config::File as GitConfig;
use std::ops::Deref;

use crate::helpers::prelude::*;

#[test]
fn it_allows_a_git_branch_to_be_specified() {
    let template = tempdir().init_default_template().branch("bak").build();
    let dir = tempdir().build();

    binary()
        .arg_branch("bak")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_allows_a_git_tag_to_be_specified() {
    let template = tempdir().init_default_template().tag("v1.0").build();
    let dir = tempdir().build();

    binary()
        .arg("--tag")
        .arg("v1.0")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_allows_a_git_revision_to_be_specified() {
    let template = tempdir().init_default_template().build();
    let commit_sha = template.commit_shas().first().unwrap().to_string();
    let dir = tempdir().build();

    binary()
        .arg_revision(commit_sha)
        .arg_git(template.path())
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_removes_git_history() {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let repo = Repository::open(dir.path().join("foobar-project")).unwrap();
    let references = repo.references().unwrap().count();
    assert_eq!(0, references);
}

#[test]
fn it_removes_git_history_also_on_local_templates() {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();

    binary()
        .arg_path(template.path())
        .arg_name("xyz")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let target_path = dir.target_path("xyz");
    let repo = git2::Repository::open(target_path).unwrap();
    assert_eq!(0, repo.references().unwrap().count());
}

#[test]
fn it_should_init_an_empty_git_repo_even_when_starting_from_a_repo_when_forced() {
    let template = tempdir().init_default_template().build();
    let target = tempdir()
        .file("README", "any existing file on a git repo")
        .init_git()
        .build();

    // the target path is a git repo with commits
    let repo = Repository::open(target.path()).unwrap();
    let references = repo.references().unwrap().count();
    assert_ne!(0, references);

    binary()
        .arg("--force-git-init")
        .arg_git(template.path())
        .arg_name("foo")
        .current_dir(target.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(target.read("foo/Cargo.toml").contains(r#"name = "foo""#));
    assert!(target.exists("foo/.git"));

    // the generated project should be an empty git repo, even if the target path was a git repo already
    let repo = Repository::open(target.path().join("foo")).unwrap();
    let references = repo.references().unwrap().count();
    assert_eq!(0, references);
}

#[test]
fn should_retrieve_an_instead_of_url() {
    let input = r#"
[url "ssh://git@github.com:"]
    insteadOf = https://github.com/
"#;
    let mut config = GitConfig::try_from(input).unwrap();
    let url = config
        .string_by("url", Some("ssh://git@github.com:".into()), "insteadOf")
        .unwrap();
    assert_eq!(url.deref(), "https://github.com/");
    config
        .set_raw_value_by(
            "url",
            Some("ssh://git@github.com:".into()),
            "insteadOf",
            "foo",
        )
        .unwrap();
}

#[test]
fn should_find_them_all() {
    let input = r#"
[url "ssh://git@github.com:"]
    insteadOf = https://github.com/
[url "ssh://git@bitbucket.org:"]
    insteadOf = https://bitbucket.org/
"#;
    let config = GitConfig::try_from(input).unwrap();
    let url = config.sections_by_name("url").unwrap();
    assert_eq!(config.sections_by_name("url").unwrap().count(), 2);

    for section in url {
        let head = section.header();
        let body = section.body();

        let url = head.subsection_name().as_ref().unwrap().to_str().unwrap();

        let instead_of_value = body.value("insteadOf").unwrap();
        let instead_of = instead_of_value.to_str().unwrap();
        if instead_of.contains("github") {
            assert_eq!(url, "ssh://git@github.com:");
            assert_eq!(instead_of, "https://github.com/")
        } else {
            assert_eq!(url, "ssh://git@bitbucket.org:");
            assert_eq!(instead_of, "https://bitbucket.org/")
        }
    }
}
