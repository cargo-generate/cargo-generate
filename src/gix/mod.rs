//! gix-based replacement for the git2 clone path.
//!
//! Public API surface intentionally mirrors `crate::git::clone_tool::RepoCloneBuilder`
//! so migrating call sites is a type-swap.

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use console::style;
use gix::prepare_clone;
use gix::remote::fetch::Shallow;
use gix::url;
use log::{debug, info};

use crate::emoji::WRENCH;
use crate::git::{gitconfig, remove_history, utils};

type BranchName = String;

/// Which target the checkout should point to after clone.
enum CheckoutTarget {
    /// A partial ref name like `main` or `feat/one` — resolved as a branch/tag on the remote.
    Ref(String),
    /// A full object ID or a fully-qualified reference (`refs/…`). Fetched as a detached HEAD.
    Revision(String),
}

pub struct RepoCloneBuilder {
    url: String,
    identity_file: Option<PathBuf>,
    target: Option<CheckoutTarget>,
    skip_submodules: bool,
    requires_full_history: bool,
    destination_path: Option<PathBuf>,
}

impl RepoCloneBuilder {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            identity_file: None,
            target: None,
            skip_submodules: false,
            requires_full_history: false,
            destination_path: None,
        }
    }

    pub const fn with_submodules(mut self, with_submodules: bool) -> Self {
        self.skip_submodules = !with_submodules;
        self
    }

    /// Uses `gitcfg` (or auto-discovered `~/.gitconfig`) to rewrite the clone url
    /// through any matching `[url "…"] insteadOf` entries.
    pub fn with_gitconfig(mut self, gitcfg: Option<&Path>) -> Result<Self> {
        if let Some(gitconfig) = gitcfg
            .map(|p| p.to_owned())
            .or_else(|| gitconfig::find_gitconfig().ok().flatten())
        {
            if let Some(url) = gitconfig::resolve_instead_url(&self.url, gitconfig)? {
                debug!("{} gitconfig 'insteadOf' lead to this url: {}", WRENCH, url);
                self.url = url;
            }
        }
        Ok(self)
    }

    pub fn with_ssh_identity(mut self, identity_path: Option<&Path>) -> Result<Self> {
        if let Some(identity_path) = identity_path {
            let identity_path = utils::canonicalize_path(identity_path)?;
            info!(
                "{} `{}` {}",
                style("Using private key:").bold(),
                style(format_args!("{}", identity_path.display()))
                    .bold()
                    .yellow(),
                style("for git-ssh checkout").bold()
            );
            self.identity_file = Some(identity_path);
        }
        Ok(self)
    }

    pub fn with_branch(mut self, branch: Option<&str>) -> Self {
        if let Some(branch) = branch {
            self.target = Some(CheckoutTarget::Ref(branch.to_owned()));
        }
        self
    }

    /// Ensures a specific tag is cloned. Overrides a previously-set revision.
    pub fn with_tag(mut self, tag: Option<&str>) -> Self {
        if let Some(tag) = tag {
            self.target = Some(CheckoutTarget::Ref(tag.to_owned()));
            self.requires_full_history = false;
        }
        self
    }

    /// Ensures a specific revision is cloned. Overrides a previously-set tag.
    pub fn with_revision(mut self, revision: Option<&str>) -> Self {
        if let Some(revision) = revision {
            self.target = Some(CheckoutTarget::Revision(revision.to_owned()));
            self.requires_full_history = true;
        }
        self
    }

    pub fn with_destination(mut self, destination_path: impl AsRef<Path>) -> Result<Self> {
        self.destination_path = Some(utils::canonicalize_path(destination_path.as_ref())?);
        Ok(self)
    }

    pub fn build(self) -> Result<GitCloneCmd> {
        if self.destination_path.is_none() {
            anyhow::bail!("Destination path is not set");
        }
        Ok(GitCloneCmd { builder: self })
    }
}

pub struct GitCloneCmd {
    builder: RepoCloneBuilder,
}

impl GitCloneCmd {
    /// Clones the configured repository and returns the checked-out branch's short name
    /// (or the object id, if HEAD ended up detached).
    pub fn do_clone(self) -> Result<BranchName> {
        let RepoCloneBuilder {
            url: url_str,
            identity_file,
            target,
            skip_submodules,
            requires_full_history,
            destination_path,
        } = self.builder;
        let dest = destination_path.expect("build() enforces destination is set");

        let url =
            url::parse(url_str.as_str()).with_context(|| format!("Invalid git url: {url_str}"))?;

        // gix's `with_revision` only accepts a full 40-char SHA (or a `refs/…`).
        // Short SHAs — which `git`, `git2`, and cargo-generate's users all accept —
        // are resolved here via a bare peek clone.
        let target = match target {
            Some(CheckoutTarget::Revision(rev)) if looks_like_short_sha(&rev) => {
                let full = peek_resolve_short_sha(&url, &rev, identity_file.as_deref())?;
                Some(CheckoutTarget::Revision(full))
            }
            other => other,
        };

        let mut prepare_clone = prepare_clone(url.clone(), &dest)
            .context("Please check if the Git user / repository exists.")?;

        prepare_clone = match target {
            Some(CheckoutTarget::Ref(name)) => prepare_clone.with_ref_name(Some(name.as_str()))?,
            Some(CheckoutTarget::Revision(rev)) => prepare_clone.with_revision(Some(rev))?,
            None => prepare_clone,
        };

        if should_limit_fetch_depth(&url_str, requires_full_history) {
            let depth = NonZeroU32::new(1).expect("1 is non-zero");
            prepare_clone = prepare_clone.with_shallow(Shallow::DepthAtRemote(depth));
        }

        if let Some(identity) = identity_file.as_deref() {
            // gix has no in-process ssh stack — it shells out to `ssh`.
            // Setting core.sshCommand mirrors `GIT_SSH_COMMAND=ssh -i <key>` for real git.
            prepare_clone = prepare_clone.with_in_memory_config_overrides([format!(
                "core.sshCommand=ssh -i {}",
                sh_single_quote(&identity.display().to_string())
            )]);
        }

        let (mut prepare_checkout, _) = prepare_clone
            .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .context("Please check if the Git user / repository exists.")?;

        let (repo, _) = prepare_checkout
            .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .context("Checkout of worktree failed")?;

        if !skip_submodules {
            init_and_update_submodules(&repo, &dest, &url, identity_file.as_deref())?;
        }

        let branch = match repo.head_name()? {
            Some(name) => name.shorten().to_string(),
            None => repo.head_id()?.to_string(),
        };

        // Templates must not carry the source repository's history.
        remove_history(&dest)?;

        Ok(branch)
    }
}

/// Initialize + update submodules recursively using gix only.
///
/// For each active submodule we clone its url pinned at the SHA the superproject
/// recorded (via `Submodule::head_id()`), recurse into it, then strip its `.git`
/// dir — the template semantics of cargo-generate never want inner history.
///
/// gix 0.87 has no high-level submodule init/update API (see gitoxide#2387);
/// this is our own take. The parent's ssh identity override is inherited so
/// private submodules work when the parent clone did.
fn init_and_update_submodules(
    super_repo: &gix::Repository,
    super_worktree: &Path,
    super_url: &gix::Url,
    identity: Option<&Path>,
) -> Result<()> {
    let Some(submodules) = super_repo.submodules()? else {
        return Ok(());
    };

    for sub in submodules {
        // Deliberately skip `sub.is_active()` — that checks `.git/config`, which is
        // empty right after clone (`git submodule init` is what populates it).
        // Templates want every declared submodule initialized.
        let sub_name = sub.name().to_string();
        let sub_rel_path = gix::path::from_bstring(sub.path()?);
        let sub_abs_path = super_worktree.join(&sub_rel_path);
        let sub_url = resolve_submodule_url(super_url, sub.url()?);
        let pinned = sub.head_id().ok().flatten();

        let mut prepare = prepare_clone(sub_url.clone(), &sub_abs_path)
            .with_context(|| format!("Failed to prepare submodule '{sub_name}'"))?;
        if let Some(sha) = pinned {
            prepare = prepare
                .with_revision(Some(sha.to_string()))
                .with_context(|| format!("Failed to pin submodule '{sub_name}' to {sha}"))?;
        }
        if let Some(identity) = identity {
            prepare = prepare.with_in_memory_config_overrides([format!(
                "core.sshCommand=ssh -i {}",
                sh_single_quote(&identity.display().to_string())
            )]);
        }

        let (mut checkout, _) = prepare
            .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .with_context(|| format!("Failed to fetch submodule '{sub_name}'"))?;
        let (sub_repo, _) = checkout
            .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .with_context(|| format!("Failed to checkout submodule '{sub_name}'"))?;

        // Recurse *before* stripping .git so nested submodules can be read.
        init_and_update_submodules(&sub_repo, &sub_abs_path, &sub_url, identity)?;

        remove_history(&sub_abs_path)
            .with_context(|| format!("Failed to strip .git from submodule '{sub_name}'"))?;
    }

    Ok(())
}

/// Resolve a submodule url written as `./foo` or `../bar` against the parent's origin.
/// Non-relative urls (http/ssh/git/file with an absolute path) are returned unchanged.
///
/// `gix_url` parses relative paths as `Scheme::File` with the raw path preserved, so we
/// detect the relative shape from there and manipulate the parent's serialized url form
/// with git's classic "pop last path component per `../`" rule.
fn resolve_submodule_url(parent: &gix::Url, sub: gix::Url) -> gix::Url {
    if sub.scheme != gix::url::Scheme::File {
        return sub;
    }
    let sub_path = sub.path.to_string();
    if !sub_path.starts_with("./") && !sub_path.starts_with("../") {
        return sub;
    }
    let parent_str = parent.to_bstring().to_string();
    let joined = join_relative_url(&parent_str, &sub_path);
    gix::url::parse(joined.as_str()).unwrap_or(sub)
}

fn join_relative_url(parent: &str, rel: &str) -> String {
    let mut base = parent.trim_end_matches('/').to_owned();
    let mut rest = rel;
    while let Some(stripped) = rest.strip_prefix("../") {
        if let Some(pos) = base.rfind('/') {
            base.truncate(pos);
        }
        rest = stripped;
    }
    if let Some(stripped) = rest.strip_prefix("./") {
        rest = stripped;
    }
    format!("{base}/{rest}")
}

/// A short SHA is 1..40 lowercase-hex chars — anything else (`HEAD`, `refs/…`,
/// full 40-char SHA) is passed straight to gix.
fn looks_like_short_sha(s: &str) -> bool {
    !s.is_empty() && s.len() < 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Resolve a short SHA to its full 40-char form by doing a bare peek clone in a
/// tempdir. gix's `PrepareFetch::with_revision` rejects anything that isn't a
/// full SHA (or `refs/…`), so we can't ask it to expand for us.
fn peek_resolve_short_sha(url: &gix::Url, short: &str, identity: Option<&Path>) -> Result<String> {
    let scratch =
        tempfile::tempdir().context("Failed to create scratch dir for revision resolve")?;
    let mut prepare = gix::prepare_clone_bare(url.clone(), scratch.path())
        .context("Failed to prepare peek clone for short-SHA resolution")?;
    if let Some(identity) = identity {
        prepare = prepare.with_in_memory_config_overrides([format!(
            "core.sshCommand=ssh -i {}",
            sh_single_quote(&identity.display().to_string())
        )]);
    }
    let (repo, _) = prepare
        .fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .with_context(|| format!("Peek clone to resolve revision '{short}' failed"))?;
    let id = repo
        .rev_parse_single(short)
        .with_context(|| format!("Revision '{short}' could not be resolved on the remote"))?;
    Ok(id.detach().to_string())
}

fn is_http_repo_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn should_limit_fetch_depth(url: &str, requires_full_history: bool) -> bool {
    is_http_repo_url(url) && !requires_full_history
}

/// Look up the branch of the git repo at `path`. Returns `None` if the path
/// is not a repository or HEAD is detached / unborn.
pub fn try_get_branch_from_path(path: impl AsRef<Path>) -> Option<String> {
    let repo = gix::open(path.as_ref()).ok()?;
    let name = repo.head_name().ok().flatten()?;
    Some(name.shorten().to_string())
}

/// POSIX single-quote escape: wrap in `'…'`, split-escape any embedded `'`.
/// The `sh -c` interpreter that gix uses for `core.sshCommand` will unquote this back.
fn sh_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use crate::git::tmp_dir;

    use super::*;
    use std::fs::metadata;

    #[test]
    fn test_cloning_a_repo() {
        let dst = tmp_dir().unwrap();

        let branch = RepoCloneBuilder::new("https://github.com/cargo-generate/cargo-generate.git")
            .with_destination(dst.path())
            .unwrap()
            .build()
            .unwrap()
            .do_clone()
            .unwrap();

        assert_eq!(branch, "main");
        assert!(metadata(dst.path().join(".git")).is_err());
    }

    #[test]
    fn test_cloning_a_repo_at_revision() {
        let dst = tmp_dir().unwrap();

        let branch = RepoCloneBuilder::new("https://github.com/cargo-generate/cargo-generate.git")
            .with_revision(Some("65748e97b43a5aadd4b34042881c80637c97a30b"))
            .with_destination(dst.path())
            .unwrap()
            .build()
            .unwrap()
            .do_clone()
            .unwrap();

        assert_eq!(branch, "65748e97b43a5aadd4b34042881c80637c97a30b");
        assert!(metadata(dst.path().join(".git")).is_err());
    }

    #[test]
    fn test_cloning_a_repo_with_a_specific_branch() {
        let dst = tmp_dir().unwrap();

        let branch = RepoCloneBuilder::new("https://github.com/cargo-generate/cargo-generate.git")
            .with_branch(Some("feat/1037-gix-as-git2-successor"))
            .with_destination(dst.path())
            .unwrap()
            .build()
            .unwrap()
            .do_clone()
            .unwrap();

        assert_eq!(branch, "feat/1037-gix-as-git2-successor");
        assert!(metadata(dst.path().join(".git")).is_err());
    }

    #[test]
    fn build_requires_destination() {
        let err = match RepoCloneBuilder::new("https://github.com/example/template.git").build() {
            Ok(_) => panic!("expected build() to fail without a destination"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("Destination"));
    }

    #[test]
    fn sh_single_quote_wraps_plain_paths() {
        assert_eq!(
            sh_single_quote("/home/user/.ssh/id_ed25519"),
            "'/home/user/.ssh/id_ed25519'"
        );
    }

    #[test]
    fn sh_single_quote_escapes_embedded_single_quote() {
        assert_eq!(sh_single_quote("/path/it's/key"), "'/path/it'\\''s/key'");
    }

    #[test]
    fn sh_single_quote_preserves_spaces() {
        assert_eq!(
            sh_single_quote("/home/some user/id_rsa"),
            "'/home/some user/id_rsa'"
        );
    }

    #[test]
    fn http_clones_are_shallow_by_default() {
        assert!(should_limit_fetch_depth(
            "https://github.com/example/template",
            false
        ));
    }

    #[test]
    fn revision_clones_skip_shallow_http_fetch() {
        assert!(!should_limit_fetch_depth(
            "https://github.com/example/template",
            true
        ));
    }

    #[test]
    fn non_http_clones_do_not_set_fetch_depth() {
        assert!(!should_limit_fetch_depth("git@example.com:repo.git", false));
    }

    #[test]
    fn join_relative_url_appends_dot_slash() {
        assert_eq!(
            join_relative_url("https://github.com/foo/bar.git", "./baz.git"),
            "https://github.com/foo/bar.git/baz.git"
        );
    }

    #[test]
    fn join_relative_url_walks_up_with_dot_dot_slash() {
        assert_eq!(
            join_relative_url("https://github.com/foo/bar.git", "../baz.git"),
            "https://github.com/foo/baz.git"
        );
    }

    #[test]
    fn join_relative_url_walks_up_twice() {
        assert_eq!(
            join_relative_url("https://github.com/foo/bar/child.git", "../../shared.git"),
            "https://github.com/foo/shared.git"
        );
    }

    #[test]
    fn join_relative_url_handles_file_path_parents() {
        assert_eq!(
            join_relative_url("/tmp/parent-repo", "../shared.git"),
            "/tmp/shared.git"
        );
    }
}
