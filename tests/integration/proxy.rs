//! End-to-end tests for [`gitconfig`] `http.proxy` support.
//!
//! Each test writes a scratch `gitconfig` file with `http.proxy` pointing at a
//! [`FakeProxy`] listening on a random localhost port, invokes `cargo-generate`
//! against a dummy git URL, and asserts the fake proxy received a connection.
//!
//! The clone always fails (the fake proxy closes the socket immediately) —
//! that is expected. The assertion is that the connection was *routed through
//! the proxy*, not that it succeeded. Every proxy-related env var is scrubbed
//! from the subprocess so a developer's shell state cannot poison the result.

use crate::helpers::prelude::*;
use indoc::formatdoc;

const DUMMY_GIT_URL: &str = "https://github.com/cargo-generate/does-not-exist.git";

const PROXY_ENV_KEYS: &[&str] = &[
    "HTTP_PROXY",
    "http_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
    "NO_PROXY",
    "no_proxy",
];

fn write_gitconfig_with_proxy(scheme: &str, authority: &str) -> Project {
    tempdir()
        .file(
            ".gitconfig",
            formatdoc! { r#"
                [http]
                    proxy = {scheme}://{authority}
            "# },
        )
        .build()
}

fn run_generate(gitconfig: PathBuf, cwd: PathBuf) -> assert_cmd::assert::Assert {
    let mut cmd = binary();
    cmd.arg_gitconfig(gitconfig)
        .arg_git(DUMMY_GIT_URL)
        .arg_name("proxy-test");
    let command = cmd.current_dir(cwd);
    for key in PROXY_ENV_KEYS {
        command.env_remove(key);
    }
    command.assert()
}

#[test]
fn http_proxy_in_gitconfig_is_honored() {
    let proxy = FakeProxy::spawn();
    let gitconfig_dir = write_gitconfig_with_proxy("http", &proxy.authority());
    let target = tempdir().build();

    run_generate(
        gitconfig_dir.path().join(".gitconfig"),
        target.path().to_path_buf(),
    )
    .failure();

    assert!(
        proxy.was_hit(),
        "cargo-generate did not route through http.proxy = http://{} — the gitconfig proxy shim is not wiring up",
        proxy.authority()
    );
}

#[test]
fn https_proxy_in_gitconfig_is_honored() {
    let proxy = FakeProxy::spawn();
    let gitconfig_dir = write_gitconfig_with_proxy("https", &proxy.authority());
    let target = tempdir().build();

    run_generate(
        gitconfig_dir.path().join(".gitconfig"),
        target.path().to_path_buf(),
    )
    .failure();

    assert!(
        proxy.was_hit(),
        "cargo-generate did not route through http.proxy = https://{} — the gitconfig proxy shim is not wiring up",
        proxy.authority()
    );
}

#[test]
fn socks5_proxy_in_gitconfig_is_honored() {
    let proxy = FakeProxy::spawn();
    let gitconfig_dir = write_gitconfig_with_proxy("socks5", &proxy.authority());
    let target = tempdir().build();

    run_generate(
        gitconfig_dir.path().join(".gitconfig"),
        target.path().to_path_buf(),
    )
    .failure();

    assert!(
        proxy.was_hit(),
        "cargo-generate did not route through http.proxy = socks5://{} — either the shim is not wiring up, \
         or reqwest was not built with the `socks` feature",
        proxy.authority()
    );
}
