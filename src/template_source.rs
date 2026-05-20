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
}
