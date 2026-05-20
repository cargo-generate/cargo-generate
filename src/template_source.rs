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
}
