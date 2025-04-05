//! https://github.com/GitoxideLabs/gitoxide/issues/301

use anyhow::Result;
use gix::prepare_clone;
use gix::refspec::parse::Operation;
use gix::remote::ref_map::Options;
use gix::url::{self, Url};
use std::path::{Path, PathBuf};

use crate::git::{gitconfig, remove_history};

type BranchName = String;

/// aiming for the same
struct RepoCloneBuilderImpl {
    url: Url,
    branch: Option<BranchName>,
    identity_file: Option<PathBuf>,
}

impl RepoCloneBuilderImpl {
    pub fn new(url: &str) -> Result<Self> {
        let repo_url = gitconfig::find_gitconfig()?.map_or_else(
            || url.to_owned(),
            |gitcfg| {
                gitconfig::resolve_instead_url(url, gitcfg)
                    .expect("correct configuration")
                    .unwrap_or_else(|| url.to_owned())
            },
        );

        let url = url::parse(repo_url.as_str().into())?;

        Ok(Self {
            url,
            branch: None,
            identity_file: None,
        })
    }

    pub fn with_branch(mut self, branch_name: Option<impl Into<BranchName>>) -> Self {
        if let Some(branch_name) = branch_name {
            self.branch = Some(format!("refs/heads/{}", branch_name.into()));
        }

        self
    }

    /// Sets the revision by git sha to checkout, like `189be32ab06134794a573ef6e74bf9a7cc0abc61`
    pub fn with_revision(mut self, revision: impl Into<Option<String>>) -> Self {
        if let Some(revision) = revision.into() {
            // todo validation for git sha length and format
            self.branch = Some(format!("{}", revision));
        }
        self
    }

    pub fn with_identity_file(mut self, identity_file: Option<impl AsRef<Path>>) -> Self {
        if let Some(identity_file) = identity_file {
            self.identity_file = Some(identity_file.as_ref().to_path_buf())
        }

        self
    }

    /// performs a git clone operation
    pub fn checkout(self, dest_path: &Path) -> Result<BranchName> {
        let mut prepare_clone = prepare_clone(self.url, dest_path)?;

        let (mut prepare_checkout, _) = if let Some(branch) = self.branch {
            let mut opts = Options::default();
            let ref_spec = gix::refspec::parse(branch.as_str().into(), Operation::Fetch).unwrap();
            dbg!(ref_spec);
            opts.extra_refspecs.push(ref_spec.to_owned());

            prepare_clone.with_fetch_options(opts)
        } else {
            prepare_clone
        }
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

        let (repo, _) = prepare_checkout
            .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

        if let Some(sub_mod) = repo.submodules()? {
            for sub in sub_mod {
                let _ = sub.fetch_recurse()?;
            }
        }

        let branch = repo.head_name()?.unwrap().shorten().to_string();

        // todo: refactor code so that remove_history
        remove_history(dest_path)?;

        Ok(branch)
    }
}

#[cfg(test)]
mod tests {
    use crate::git::tmp_dir;

    use super::*;
    use std::fs::metadata;

    #[test]
    fn test_cloning_a_repo() {
        let dst = tmp_dir().unwrap();
        let repo_url = "https://github.com/cargo-generate/cargo-generate.git";

        let branch = RepoCloneBuilderImpl::new(repo_url)
            .unwrap()
            .checkout(dst.path())
            .unwrap();

        assert_eq!(branch, "main");
        assert!(metadata(dst.path().join(".git")).is_err());
    }

    #[test]
    fn test_cloning_a_repo_at_revision() {
        let dst = tmp_dir().unwrap();
        let repo_url = "https://github.com/cargo-generate/cargo-generate.git";

        let branch = RepoCloneBuilderImpl::new(repo_url)
            .unwrap()
            .with_revision("65748e97b43a5aadd4b34042881c80637c97a30b".to_string())
            .checkout(dst.path())
            .unwrap();

        assert_eq!(branch, "65748e97b43a5aadd4b34042881c80637c97a30b");
        assert!(metadata(dst.path().join(".git")).is_err());
    }

    #[test]
    fn test_cloning_a_repo_with_a_specific_branch() {
        let dst = tmp_dir().unwrap();
        dbg!(&dst.path());

        let repo_url = "https://github.com/cargo-generate/cargo-generate.git";

        let branch = RepoCloneBuilderImpl::new(repo_url)
            .unwrap()
            .with_branch("feat/1037-gix-as-git2-successor".into())
            .checkout(dst.path())
            .unwrap();

        assert_eq!(branch, "feat/1037-gix-as-git2-successor");
        assert!(metadata(dst.path().join(".git")).is_err());
    }

    #[test]
    fn test_new_clone_repo_builder() {
        assert!(
            RepoCloneBuilderImpl::new("https://github.com/cargo-generate/cargo-generate.git")
                .is_ok()
        );
        assert!(
            RepoCloneBuilderImpl::new("git@github.com:cargo-generate/cargo-generate.git").is_ok()
        );
        assert!(RepoCloneBuilderImpl::new(
            "/Users/I563162/workspace/cargo-generate/cargo-generate"
        )
        .is_ok());
    }
}
