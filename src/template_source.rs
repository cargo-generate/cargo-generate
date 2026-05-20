//! Classified template source: where the template comes from, validated
//! at construction time. See
//! `docs/superpowers/specs/2026-05-20-template-source-classifier-design.md`.

use std::path::PathBuf;

/// Options threaded into git clones for remote sources. Local sources
/// ignore these. Used by `TemplateSource::into_template_location`.
#[derive(Debug, Clone, Default)]
pub struct CloneOptions {
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub revision: Option<String>,
    pub ssh_identity: Option<PathBuf>,
    pub gitconfig: Option<PathBuf>,
    pub force_git_init: bool,
    pub skip_submodules: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GitHost {
    GitHub,
    GitLab,
    Bitbucket,
    SourceHut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TemplateSource {
    HostShorthand { host: GitHost, owner_repo: String },
    GithubOwnerRepo { owner: String, repo: String },
    RemoteUrl(String),
    LocalRelative(PathBuf),
    LocalAbsolute(PathBuf),
    Favorite(Box<TemplateSource>),
}

impl GitHost {
    pub fn to_url(&self, owner_repo: &str) -> String {
        match self {
            Self::GitHub => format!("https://github.com/{owner_repo}.git"),
            Self::GitLab => format!("https://gitlab.com/{owner_repo}.git"),
            Self::Bitbucket => format!("https://bitbucket.org/{owner_repo}.git"),
            Self::SourceHut => format!("https://git.sr.ht/~{owner_repo}"),
        }
    }
}

fn strip_host_prefix(s: &str) -> Option<(GitHost, &str)> {
    let prefix = s.get(..3)?;
    let rest = s.get(3..)?;
    let host = match prefix {
        "gh:" => GitHost::GitHub,
        "gl:" => GitHost::GitLab,
        "bb:" => GitHost::Bitbucket,
        "sr:" => GitHost::SourceHut,
        _ => return None,
    };
    Some((host, rest))
}

fn looks_like_url(s: &str) -> bool {
    if s.starts_with("https://")
        || s.starts_with("http://")
        || s.starts_with("ssh://")
        || s.starts_with("git://")
        || s.starts_with("file://")
    {
        return true;
    }
    // scp-style `git@host:path`: a `user@host:rest` form where the colon
    // is BEFORE the first slash (URLs and paths put `/` before `:`).
    if let Some(at) = s.find('@') {
        if let Some(colon_offset) = s[at..].find(':') {
            let colon = at + colon_offset;
            let slash = s.find('/').unwrap_or(s.len());
            if colon < slash {
                return true;
            }
        }
    }
    false
}

fn parse_owner_repo(s: &str) -> Option<(String, String)> {
    // Require exactly one `/`, both sides nonempty, owner side doesn't
    // start with `.` (which would be a relative path form).
    let (owner, repo) = s.split_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    if repo.contains('/') {
        return None;
    }
    if owner.starts_with('.') {
        return None;
    }
    let owner_ok = owner
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    let repo_ok = repo
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if owner_ok && repo_ok {
        Some((owner.to_owned(), repo.to_owned()))
    } else {
        None
    }
}

impl TemplateSource {
    /// Classify a raw template input into a `TemplateSource`. See the
    /// design doc for the precedence rules; ordering here is intentional.
    pub fn classify(
        input: &str,
        app_config: &crate::app_config::AppConfig,
        cwd: &std::path::Path,
    ) -> Self {
        Self::classify_with_depth(input, app_config, cwd, 0)
    }

    fn classify_with_depth(
        input: &str,
        app_config: &crate::app_config::AppConfig,
        cwd: &std::path::Path,
        depth: u8,
    ) -> Self {
        const FAVORITE_RECURSION_LIMIT: u8 = 8;

        // 1. Configured favorite name (only when within recursion budget).
        if depth <= FAVORITE_RECURSION_LIMIT {
            if let Some(fav) = app_config.get_favorite_cfg(input) {
                if let Some(git) = fav.git.as_deref() {
                    let inner = Self::classify_with_depth(git, app_config, cwd, depth + 1);
                    return Self::Favorite(Box::new(inner));
                }
                if let Some(p) = fav.path.as_ref() {
                    let inner = if p.is_absolute() {
                        Self::LocalAbsolute(p.clone())
                    } else {
                        Self::LocalRelative(cwd.join(p))
                    };
                    return Self::Favorite(Box::new(inner));
                }
                // Malformed favorite (neither git nor path) — fall through to
                // non-favorite classification rules below.
            }
        }

        // 2. Host prefix
        if let Some((host, rest)) = strip_host_prefix(input) {
            return Self::HostShorthand {
                host,
                owner_repo: rest.to_owned(),
            };
        }
        // 3. Full URL
        if looks_like_url(input) {
            return Self::RemoteUrl(input.to_owned());
        }
        // 4. Absolute path
        let p = std::path::Path::new(input);
        if p.is_absolute() {
            return Self::LocalAbsolute(p.to_path_buf());
        }
        // 5. Relative path that exists as a directory
        let resolved = cwd.join(p);
        if resolved.is_dir() {
            return Self::LocalRelative(resolved);
        }
        // 6. Bare owner/repo → github
        if let Some((owner, repo)) = parse_owner_repo(input) {
            return Self::GithubOwnerRepo { owner, repo };
        }
        // 7. Catch-all — let git produce the clearer error.
        Self::RemoteUrl(input.to_owned())
    }

    /// User-facing label preserving the form the user typed. Used in
    /// warnings, log lines, error messages.
    pub fn display_label(&self) -> std::borrow::Cow<'_, str> {
        use std::borrow::Cow;
        match self {
            Self::HostShorthand { host, owner_repo } => {
                let prefix = match host {
                    GitHost::GitHub => "gh",
                    GitHost::GitLab => "gl",
                    GitHost::Bitbucket => "bb",
                    GitHost::SourceHut => "sr",
                };
                Cow::Owned(format!("{prefix}:{owner_repo}"))
            }
            Self::GithubOwnerRepo { owner, repo } => Cow::Owned(format!("{owner}/{repo}")),
            Self::RemoteUrl(url) => Cow::Borrowed(url.as_str()),
            Self::LocalRelative(p) | Self::LocalAbsolute(p) => {
                Cow::Owned(p.display().to_string())
            }
            Self::Favorite(inner) => Cow::Owned(format!("favorite → {}", inner.display_label())),
        }
    }

    /// Adapter producing the legacy `TemplateLocation`. `clone_opts` are
    /// threaded through to `GitUserInput::new` for remote variants;
    /// ignored for local variants. Once consumers migrate, this method
    /// goes away.
    pub fn into_template_location(
        self,
        clone_opts: &CloneOptions,
    ) -> crate::user_parsed_input::TemplateLocation {
        use crate::user_parsed_input::{GitUserInput, TemplateLocation};
        match self {
            Self::HostShorthand { host, owner_repo } => TemplateLocation::Git(
                GitUserInput::with_url_and_clone_opts(host.to_url(&owner_repo), clone_opts),
            ),
            Self::GithubOwnerRepo { owner, repo } => TemplateLocation::Git(
                GitUserInput::with_url_and_clone_opts(
                    format!("https://github.com/{owner}/{repo}.git"),
                    clone_opts,
                ),
            ),
            Self::RemoteUrl(url) => TemplateLocation::Git(
                GitUserInput::with_url_and_clone_opts(url, clone_opts),
            ),
            Self::LocalAbsolute(p) | Self::LocalRelative(p) => TemplateLocation::Path(p),
            Self::Favorite(inner) => inner.into_template_location(clone_opts),
        }
    }

    #[cfg(test)]
    fn into_template_location_for_test(self) -> crate::user_parsed_input::TemplateLocation {
        self.into_template_location(&CloneOptions::default())
    }

    /// Whether this source should be acquired by cloning vs copying.
    /// Favorites delegate to their inner source.
    pub fn is_remote(&self) -> bool {
        match self {
            Self::HostShorthand { .. } | Self::GithubOwnerRepo { .. } | Self::RemoteUrl(_) => true,
            Self::LocalRelative(_) | Self::LocalAbsolute(_) => false,
            Self::Favorite(inner) => inner.is_remote(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::AppConfig;
    use std::path::Path;

    fn empty_config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn classify_host_shorthand() {
        let s = TemplateSource::classify("gh:owner/repo", &empty_config(), Path::new("/tmp"));
        assert_eq!(
            s,
            TemplateSource::HostShorthand {
                host: GitHost::GitHub,
                owner_repo: "owner/repo".to_owned()
            }
        );
    }

    #[test]
    fn classify_https_url() {
        let s = TemplateSource::classify(
            "https://github.com/owner/repo.git",
            &empty_config(),
            Path::new("/tmp"),
        );
        assert_eq!(
            s,
            TemplateSource::RemoteUrl("https://github.com/owner/repo.git".to_owned())
        );
    }

    #[test]
    fn classify_scp_url() {
        let s = TemplateSource::classify(
            "git@github.com:owner/repo.git",
            &empty_config(),
            Path::new("/tmp"),
        );
        assert_eq!(
            s,
            TemplateSource::RemoteUrl("git@github.com:owner/repo.git".to_owned())
        );
    }

    #[test]
    fn classify_absolute_path() {
        let s = TemplateSource::classify("/Users/me/template", &empty_config(), Path::new("/tmp"));
        assert_eq!(
            s,
            TemplateSource::LocalAbsolute(PathBuf::from("/Users/me/template"))
        );
    }

    #[test]
    fn classify_existing_relative_dir() {
        let cwd = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(cwd.path().join("template")).unwrap();
        let s = TemplateSource::classify("./template", &empty_config(), cwd.path());
        assert_eq!(s, TemplateSource::LocalRelative(cwd.path().join("template")));
    }

    #[test]
    fn classify_owner_repo_when_no_local_dir() {
        let cwd = tempfile::TempDir::new().unwrap();
        let s = TemplateSource::classify("owner/repo", &empty_config(), cwd.path());
        assert_eq!(
            s,
            TemplateSource::GithubOwnerRepo {
                owner: "owner".to_owned(),
                repo: "repo".to_owned()
            }
        );
    }

    #[test]
    fn classify_relative_dir_beats_owner_repo() {
        let cwd = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(cwd.path().join("owner/repo")).unwrap();
        let s = TemplateSource::classify("owner/repo", &empty_config(), cwd.path());
        assert_eq!(
            s,
            TemplateSource::LocalRelative(cwd.path().join("owner/repo"))
        );
    }

    #[test]
    fn classify_fallback_to_remote_url() {
        let cwd = tempfile::TempDir::new().unwrap();
        let s = TemplateSource::classify("garbage~~~", &empty_config(), cwd.path());
        assert_eq!(s, TemplateSource::RemoteUrl("garbage~~~".to_owned()));
    }

    #[test]
    fn git_host_to_url_github() {
        assert_eq!(
            GitHost::GitHub.to_url("owner/repo"),
            "https://github.com/owner/repo.git"
        );
    }

    #[test]
    fn git_host_to_url_gitlab() {
        assert_eq!(
            GitHost::GitLab.to_url("owner/repo"),
            "https://gitlab.com/owner/repo.git"
        );
    }

    #[test]
    fn git_host_to_url_bitbucket() {
        assert_eq!(
            GitHost::Bitbucket.to_url("owner/repo"),
            "https://bitbucket.org/owner/repo.git"
        );
    }

    #[test]
    fn git_host_to_url_sourcehut_inserts_tilde() {
        assert_eq!(
            GitHost::SourceHut.to_url("user/repo"),
            "https://git.sr.ht/~user/repo"
        );
    }

    #[test]
    fn strip_host_prefix_recognizes_gh() {
        assert_eq!(strip_host_prefix("gh:o/r"), Some((GitHost::GitHub, "o/r")));
    }
    #[test]
    fn strip_host_prefix_recognizes_gl() {
        assert_eq!(strip_host_prefix("gl:o/r"), Some((GitHost::GitLab, "o/r")));
    }
    #[test]
    fn strip_host_prefix_recognizes_bb() {
        assert_eq!(strip_host_prefix("bb:o/r"), Some((GitHost::Bitbucket, "o/r")));
    }
    #[test]
    fn strip_host_prefix_recognizes_sr() {
        assert_eq!(strip_host_prefix("sr:u/r"), Some((GitHost::SourceHut, "u/r")));
    }
    #[test]
    fn strip_host_prefix_rejects_unknown_prefix() {
        assert_eq!(strip_host_prefix("xx:o/r"), None);
    }
    #[test]
    fn strip_host_prefix_rejects_short_input() {
        assert_eq!(strip_host_prefix("gh"), None);
    }
    #[test]
    fn strip_host_prefix_rejects_bare_owner_repo() {
        assert_eq!(strip_host_prefix("owner/repo"), None);
    }

    #[test]
    fn looks_like_url_https() {
        assert!(looks_like_url("https://github.com/owner/repo.git"));
    }
    #[test]
    fn looks_like_url_http() {
        assert!(looks_like_url("http://example.com/repo.git"));
    }
    #[test]
    fn looks_like_url_ssh_scheme() {
        assert!(looks_like_url("ssh://git@github.com/owner/repo.git"));
    }
    #[test]
    fn looks_like_url_git_scheme() {
        assert!(looks_like_url("git://github.com/owner/repo.git"));
    }
    #[test]
    fn looks_like_url_scp_style() {
        assert!(looks_like_url("git@github.com:owner/repo.git"));
    }
    #[test]
    fn looks_like_url_rejects_owner_repo() {
        assert!(!looks_like_url("owner/repo"));
    }
    #[test]
    fn looks_like_url_rejects_relative_path() {
        assert!(!looks_like_url("./template"));
    }
    #[test]
    fn looks_like_url_rejects_absolute_path() {
        assert!(!looks_like_url("/Users/me/template"));
    }
    #[test]
    fn looks_like_url_rejects_host_prefix() {
        assert!(!looks_like_url("gh:owner/repo"));
    }

    #[test]
    fn parse_owner_repo_simple() {
        assert_eq!(
            parse_owner_repo("owner/repo"),
            Some(("owner".to_owned(), "repo".to_owned()))
        );
    }
    #[test]
    fn parse_owner_repo_with_dashes_and_dots() {
        assert_eq!(
            parse_owner_repo("my-org/my.repo"),
            Some(("my-org".to_owned(), "my.repo".to_owned()))
        );
    }
    #[test]
    fn parse_owner_repo_rejects_relative_dot_prefix() {
        assert_eq!(parse_owner_repo("./path"), None);
    }
    #[test]
    fn parse_owner_repo_rejects_double_dot() {
        assert_eq!(parse_owner_repo("../path"), None);
    }
    #[test]
    fn parse_owner_repo_rejects_three_segments() {
        assert_eq!(parse_owner_repo("a/b/c"), None);
    }
    #[test]
    fn parse_owner_repo_rejects_leading_slash() {
        assert_eq!(parse_owner_repo("/abs/path"), None);
    }
    #[test]
    fn parse_owner_repo_rejects_trailing_slash() {
        assert_eq!(parse_owner_repo("owner/"), None);
    }
    #[test]
    fn parse_owner_repo_rejects_no_slash() {
        assert_eq!(parse_owner_repo("ownerrepo"), None);
    }

    #[test]
    fn classify_malformed_favorite_falls_through_to_non_favorite() {
        use crate::app_config::FavoriteConfig;
        use std::collections::HashMap;
        let mut favorites = HashMap::new();
        // Malformed favorite: name is set but neither git nor path is.
        favorites.insert("just-a-name".to_owned(), FavoriteConfig::default());
        let cfg = AppConfig {
            favorites: Some(favorites),
            ..AppConfig::default()
        };
        let cwd = tempfile::TempDir::new().unwrap();
        let s = TemplateSource::classify("just-a-name", &cfg, cwd.path());
        // No `/`, no scheme, no local dir → catch-all RemoteUrl with the
        // verbatim input, NOT wrapped in Favorite.
        assert_eq!(s, TemplateSource::RemoteUrl("just-a-name".to_owned()));
    }

    use crate::app_config::FavoriteConfig;
    use std::collections::HashMap;

    fn config_with_favorite(name: &str, fav: FavoriteConfig) -> AppConfig {
        let mut map = HashMap::new();
        map.insert(name.to_owned(), fav);
        AppConfig {
            favorites: Some(map),
            ..AppConfig::default()
        }
    }

    #[test]
    fn classify_favorite_with_git_string_recurses() {
        let cfg = config_with_favorite(
            "myfave",
            FavoriteConfig {
                git: Some("gh:owner/repo".to_owned()),
                ..FavoriteConfig::default()
            },
        );
        let s = TemplateSource::classify("myfave", &cfg, Path::new("/tmp"));
        assert_eq!(
            s,
            TemplateSource::Favorite(Box::new(TemplateSource::HostShorthand {
                host: GitHost::GitHub,
                owner_repo: "owner/repo".to_owned(),
            }))
        );
    }

    #[test]
    fn classify_favorite_with_absolute_path() {
        use std::path::PathBuf;
        let cfg = config_with_favorite(
            "myfave",
            FavoriteConfig {
                path: Some(PathBuf::from("/abs/template")),
                ..FavoriteConfig::default()
            },
        );
        let s = TemplateSource::classify("myfave", &cfg, Path::new("/tmp"));
        assert_eq!(
            s,
            TemplateSource::Favorite(Box::new(TemplateSource::LocalAbsolute(
                PathBuf::from("/abs/template")
            )))
        );
    }

    #[test]
    fn classify_favorite_cycle_bounded_falls_back() {
        // Two favorites pointing at each other; cycle bound prevents
        // infinite recursion. After the depth limit we stop following
        // favorites and the last name is treated as a non-favorite.
        let mut map = HashMap::new();
        map.insert(
            "a".to_owned(),
            FavoriteConfig {
                git: Some("b".to_owned()),
                ..FavoriteConfig::default()
            },
        );
        map.insert(
            "b".to_owned(),
            FavoriteConfig {
                git: Some("a".to_owned()),
                ..FavoriteConfig::default()
            },
        );
        let cfg = AppConfig {
            favorites: Some(map),
            ..AppConfig::default()
        };
        let s = TemplateSource::classify("a", &cfg, Path::new("/tmp"));
        // The exact deep structure doesn't matter; what matters is that
        // classify terminates and produces something well-formed.
        assert!(matches!(s, TemplateSource::Favorite(_)));
    }

    #[test]
    fn display_label_host_shorthand() {
        let s = TemplateSource::HostShorthand {
            host: GitHost::GitHub,
            owner_repo: "o/r".to_owned(),
        };
        assert_eq!(s.display_label(), "gh:o/r");
    }
    #[test]
    fn display_label_owner_repo() {
        let s = TemplateSource::GithubOwnerRepo {
            owner: "o".to_owned(),
            repo: "r".to_owned(),
        };
        assert_eq!(s.display_label(), "o/r");
    }
    #[test]
    fn display_label_remote_url() {
        let s = TemplateSource::RemoteUrl("https://x/y".to_owned());
        assert_eq!(s.display_label(), "https://x/y");
    }
    #[test]
    fn display_label_local_relative() {
        let s = TemplateSource::LocalRelative(PathBuf::from("./t"));
        assert_eq!(s.display_label(), "./t");
    }
    #[test]
    fn display_label_local_absolute() {
        let s = TemplateSource::LocalAbsolute(PathBuf::from("/abs/t"));
        assert_eq!(s.display_label(), "/abs/t");
    }
    #[test]
    fn display_label_favorite_wraps_inner() {
        let s = TemplateSource::Favorite(Box::new(TemplateSource::GithubOwnerRepo {
            owner: "o".to_owned(),
            repo: "r".to_owned(),
        }));
        assert_eq!(s.display_label(), "favorite → o/r");
    }

    use crate::user_parsed_input::TemplateLocation;

    #[test]
    fn into_template_location_maps_host_shorthand_to_git_url() {
        let s = TemplateSource::HostShorthand {
            host: GitHost::GitHub,
            owner_repo: "o/r".to_owned(),
        };
        match s.into_template_location_for_test() {
            TemplateLocation::Git(g) => assert_eq!(g.url(), "https://github.com/o/r.git"),
            TemplateLocation::Path(_) => panic!("expected Git"),
        }
    }
    #[test]
    fn into_template_location_maps_owner_repo_to_git_url() {
        let s = TemplateSource::GithubOwnerRepo {
            owner: "o".to_owned(),
            repo: "r".to_owned(),
        };
        match s.into_template_location_for_test() {
            TemplateLocation::Git(g) => assert_eq!(g.url(), "https://github.com/o/r.git"),
            TemplateLocation::Path(_) => panic!("expected Git"),
        }
    }
    #[test]
    fn into_template_location_maps_remote_url_verbatim() {
        let s = TemplateSource::RemoteUrl("ssh://git@x/y.git".to_owned());
        match s.into_template_location_for_test() {
            TemplateLocation::Git(g) => assert_eq!(g.url(), "ssh://git@x/y.git"),
            TemplateLocation::Path(_) => panic!("expected Git"),
        }
    }
    #[test]
    fn into_template_location_maps_local_to_path() {
        let s = TemplateSource::LocalAbsolute(PathBuf::from("/abs"));
        match s.into_template_location_for_test() {
            TemplateLocation::Path(p) => assert_eq!(p, Path::new("/abs")),
            TemplateLocation::Git(_) => panic!("expected Path"),
        }
    }

    #[test]
    fn is_remote_for_each_variant() {
        assert!(TemplateSource::HostShorthand {
            host: GitHost::GitHub,
            owner_repo: "o/r".to_owned()
        }
        .is_remote());
        assert!(TemplateSource::GithubOwnerRepo {
            owner: "o".to_owned(),
            repo: "r".to_owned()
        }
        .is_remote());
        assert!(TemplateSource::RemoteUrl("x".to_owned()).is_remote());
        assert!(!TemplateSource::LocalRelative(PathBuf::from("./t")).is_remote());
        assert!(!TemplateSource::LocalAbsolute(PathBuf::from("/t")).is_remote());
        // Favorite delegates to inner
        assert!(
            TemplateSource::Favorite(Box::new(TemplateSource::RemoteUrl("x".to_owned())))
                .is_remote()
        );
    }
}
