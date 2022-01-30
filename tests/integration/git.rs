use assert_cmd::prelude::*;
use git2::Repository;
use git_config::file::GitConfig;
use git_config::parser::Key;
use git_config::values::Value;
use predicates::prelude::*;
use std::ops::Deref;

use crate::helpers::project::binary;
use crate::helpers::project_builder::tmp_dir;

#[test]
fn it_allows_a_git_branch_to_be_specified() {
    let template = tmp_dir().init_default_template().branch("bak").build();
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--branch")
        .arg("bak")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_removes_git_history() {
    let template = tmp_dir().init_default_template().build();
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

    let repo = Repository::open(&dir.path().join("foobar-project")).unwrap();
    let references = repo.references().unwrap().count();
    assert_eq!(0, references);
}

#[test]
fn it_removes_git_history_also_on_local_templates() {
    let template = tmp_dir().init_default_template().build();
    let dir = tmp_dir().build();

    binary()
        .arg("generate")
        .arg("--path")
        .arg(template.path())
        .arg("--name")
        .arg("xyz")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let target_path = dir.target_path("xyz");
    let repo = git2::Repository::open(&target_path).unwrap();
    assert_eq!(0, repo.references().unwrap().count());
}

#[test]
fn it_should_init_an_empty_git_repo_even_when_starting_from_a_repo_when_forced() {
    let template = tmp_dir().init_default_template().build();
    let target_path = template.path();

    binary()
        .arg("generate")
        .arg("--force-git-init")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foo")
        .current_dir(&target_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let repo = Repository::open(&target_path.join("foo")).unwrap();
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
        .value::<Value>("url", Some("ssh://git@github.com:"), "insteadOf")
        .unwrap();
    assert_eq!(url.to_string(), "https://github.com/");
    config
        .set_raw_value(
            "url",
            Some("ssh://git@github.com:"),
            "insteadOf",
            "foo".as_bytes().to_vec(),
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
    let url = config.sections_by_name_with_header("url");
    assert_eq!(url.len(), 2);

    for (head, body) in url.iter() {
        let url = head.subsection_name.as_ref().unwrap();

        let instead_of = body.value(&Key::from("insteadOf"));
        if url.contains("github") {
            assert_eq!(url, "ssh://git@github.com:");
            assert_eq!(
                instead_of.unwrap().deref(),
                "https://github.com/".as_bytes()
            )
        } else {
            assert_eq!(url, "ssh://git@bitbucket.org:");
            assert_eq!(
                instead_of.unwrap().as_ref(),
                "https://bitbucket.org/".as_bytes()
            )
        }
    }
}
