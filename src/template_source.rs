//! Classified template source: where the template comes from, validated
//! at construction time. See
//! `docs/superpowers/specs/2026-05-20-template-source-classifier-design.md`.

use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
