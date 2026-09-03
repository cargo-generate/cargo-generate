//! Reading the user's git configuration.
//!
//! Locates the active `.gitconfig` and resolves the settings cloning
//! honors: `url.<base>.insteadOf` rewrites and http/https/socks proxies.

use crate::utils::home;
use anyhow::Context;
use anyhow::Result;
use gix::config::{File as GitConfigParser, Source};
use std::path::{Path, PathBuf};

/// Effective proxy settings pulled from a gitconfig file.
///
/// Fields mirror the two git config keys we care about: `http.proxy` and
/// `http.noProxy`. Absent keys yield `None`; empty values are preserved as
/// `Some(String::new())` so callers can distinguish "unset" from git's
/// "explicitly disabled" convention (`http.proxy = ""`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HttpProxyConfig {
    pub proxy: Option<String>,
    pub no_proxy: Option<String>,
}

/// Look up `key` (e.g. `user.name`) via the repo config discovered from
/// `cwd`, falling back to the system + global git config.
///
/// Returns `None` when the key is unset or no config can be read.
pub fn read_config_string(key: &str, cwd: &Path) -> Option<String> {
    if let Ok(repo) = gix::discover(cwd) {
        if let Some(value) = repo.config_snapshot().string(key) {
            return Some(value.to_string());
        }
    }
    GitConfigParser::from_globals()
        .ok()
        .and_then(|file| file.string(key).map(|v| v.to_string()))
}

pub fn find_gitconfig() -> Result<Option<PathBuf>> {
    let gitconfig = home().map(|home| home.join(".gitconfig"))?;
    if gitconfig.exists() {
        return Ok(Some(gitconfig));
    }

    Ok(None)
}

/// trades urls, to replace a given repo remote url with the right on based
/// on the `[url]` section in the `~/.gitconfig`
pub fn resolve_instead_url(
    remote: impl AsRef<str>,
    gitconfig: impl AsRef<Path>,
) -> Result<Option<String>> {
    let gitconfig = gitconfig.as_ref().to_path_buf();
    let remote = remote.as_ref().to_string();
    let config = GitConfigParser::from_path_no_includes(gitconfig, Source::User)
        .context("Cannot read or parse .gitconfig")?;
    let x = config.sections_by_name("url").and_then(|iter| {
        iter.map(|section| {
            let head = section.header();
            let body = section.body();
            let url = head.subsection_name();
            let instead_of = body
                .value("insteadOf")
                .map(|x| std::str::from_utf8(&x[..]).unwrap().to_owned());
            (instead_of, url)
        })
        .filter(|(old, new)| new.is_some() && old.is_some())
        .find_map(|(old, new)| {
            let old = old.unwrap();
            let new = new.unwrap().to_string();
            remote
                .starts_with(old.as_str())
                .then(|| remote.replace(old.as_str(), new.as_str()))
        })
    });

    Ok(x)
}

/// Read `http.proxy` and `http.noProxy` from the given gitconfig file.
pub fn resolve_http_proxy(gitconfig: impl AsRef<Path>) -> Result<HttpProxyConfig> {
    let path = gitconfig.as_ref().to_path_buf();
    let config = GitConfigParser::from_path_no_includes(path, Source::User)
        .context("Cannot read or parse .gitconfig")?;

    let read = |key: &str| {
        config
            .string(key)
            .map(|v| String::from_utf8_lossy(&v).into_owned())
    };

    Ok(HttpProxyConfig {
        proxy: read("http.proxy"),
        no_proxy: read("http.noProxy"),
    })
}

#[cfg(test)]
mod test {
    use crate::tmp_dir;

    use super::*;

    #[test]
    fn should_resolve_instead_url() {
        let sample_config = r#"
[url "ssh://git@github.com:"]
    insteadOf = https://github.com/
"#;
        let where_gitconfig_lives = tmp_dir().unwrap();
        let gitconfig = where_gitconfig_lives.path().join(".gitconfig");
        std::fs::write(&gitconfig, sample_config).unwrap();

        // SSH, aka git@github.com: or ssh://git@github.com/
        let x = resolve_instead_url("https://github.com/foo/bar.git", &gitconfig).unwrap();
        assert_eq!(x.unwrap().as_str(), "ssh://git@github.com:foo/bar.git")
    }

    fn write_config(body: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tmp_dir().unwrap();
        let path = dir.path().join(".gitconfig");
        std::fs::write(&path, body).unwrap();
        (dir, path)
    }

    #[test]
    fn resolve_http_proxy_returns_none_when_unset() {
        let (_dir, cfg) = write_config("");
        let got = resolve_http_proxy(&cfg).unwrap();
        assert_eq!(got, HttpProxyConfig::default());
    }

    #[test]
    fn resolve_http_proxy_reads_http_proxy() {
        let (_dir, cfg) = write_config(
            r#"
[http]
    proxy = http://proxy.internal:3128
"#,
        );
        let got = resolve_http_proxy(&cfg).unwrap();
        assert_eq!(got.proxy.as_deref(), Some("http://proxy.internal:3128"));
        assert!(got.no_proxy.is_none());
    }

    #[test]
    fn resolve_http_proxy_reads_socks5_scheme() {
        // Regression guard for cargo-generate#664 — socks5 in http.proxy must be
        // exposed verbatim; the shim later routes it into ALL_PROXY for reqwest.
        let (_dir, cfg) = write_config(
            r#"
[http]
    proxy = socks5://127.0.0.1:1081
"#,
        );
        let got = resolve_http_proxy(&cfg).unwrap();
        assert_eq!(got.proxy.as_deref(), Some("socks5://127.0.0.1:1081"));
    }

    #[test]
    fn resolve_http_proxy_reads_no_proxy() {
        let (_dir, cfg) = write_config(
            r#"
[http]
    proxy = http://p:8080
    noProxy = localhost,127.0.0.1,.internal
"#,
        );
        let got = resolve_http_proxy(&cfg).unwrap();
        assert_eq!(got.proxy.as_deref(), Some("http://p:8080"));
        assert_eq!(
            got.no_proxy.as_deref(),
            Some("localhost,127.0.0.1,.internal")
        );
    }
}
