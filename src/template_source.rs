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
