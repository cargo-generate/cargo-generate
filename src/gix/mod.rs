//! https://github.com/GitoxideLabs/gitoxide/issues/301
//! the `gix clone` cli entry point can be found here:
//!     - https://github.com/GitoxideLabs/gitoxide/blob/main/src/plumbing/main.rs#L587

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use gix::prepare_clone;
use gix::refspec::parse::Operation;
use gix::remote::ref_map::Options;
use gix::url::{self, Url};

use crate::git::{gitconfig, remove_history};

type BranchName = String;

/// aiming for the same
struct RepoCloneBuilderGix {
    url: Url,
    branch: Option<BranchName>,
    ref_name: Option<gix::refs::PartialName>,
    identity_file: Option<PathBuf>,
}

impl RepoCloneBuilderGix {
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
            ref_name: None,
            identity_file: None,
        })
    }

    pub fn with_branch(mut self, branch_name: Option<impl Into<BranchName>>) -> Self {
        if let Some(branch_name) = branch_name {
            let branch_name = branch_name.into();
            let r = gix::refs::PartialName::try_from(branch_name.as_str()).unwrap();
            self.ref_name = Some(r);

            self.branch = Some(branch_name.to_string());
        }

        self
    }

    /// Sets the revision by git sha to checkout, like `189be32ab06134794a573ef6e74bf9a7cc0abc61`
    pub fn with_revision(mut self, revision: impl Into<Option<String>>) -> Self {
        if revision.into().is_some() {
            unimplemented!("Using a git sha aka object id is not yet supported, or not clear how it is supported by gix");
        }
        self
    }

    pub fn with_identity_file(mut self, identity_file: Option<impl AsRef<Path>>) -> Self {
        if let Some(identity_file) = identity_file {
            self.identity_file = Some(identity_file.as_ref().to_path_buf())
        }

        self
    }

    pub fn checkout(self, dest_path: &Path) -> Result<BranchName> {
        let mut cmd = gix::clone::PrepareFetch::new(
            self.url,
            dest_path,
            gix::create::Kind::WithWorktree,
            gix::create::Options::default(),
            gix::open::Options::default(),
        )?;

        if let Some(ref_name) = self.ref_name {
            let full_name_ref = ref_name.as_ref();
            cmd = cmd.with_ref_name(Some(full_name_ref))?;
        }

        let progress: gix::progress::Discard = gix::progress::Discard;
        let should_interrupt = AtomicBool::new(false);

        let (x, _outcome) = cmd.fetch_then_checkout(progress, &should_interrupt)?;

        x.persist();

        Ok(self.branch.unwrap_or_else(|| "main".to_string()))
    }

    /// performs a git clone operation
    pub fn checkout_old(self, dest_path: &Path) -> Result<BranchName> {
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
    use std::{fs::metadata, process::Command};

    #[test]
    fn test_cloning_a_repo() {
        let dst = tmp_dir().unwrap();
        let repo_url = "https://github.com/cargo-generate/cargo-generate.git";

        let branch = RepoCloneBuilderGix::new(repo_url)
            .unwrap()
            .checkout(dst.path())
            .unwrap();

        let real_branch = get_repo_branch_at_path(dst.path()).unwrap();

        assert_eq!(real_branch, "main");
        assert_eq!(branch, real_branch);
        assert!(metadata(dst.path().join(".git")).is_ok());
    }

    #[test]
    #[ignore]
    fn test_cloning_a_repo_at_revision() {
        let dst = tmp_dir().unwrap();
        let repo_url = "https://github.com/cargo-generate/cargo-generate.git";

        let branch = RepoCloneBuilderGix::new(repo_url)
            .unwrap()
            .with_revision("65748e97b43a5aadd4b34042881c80637c97a30b".to_string())
            .checkout(dst.path())
            .unwrap();

        assert_eq!(branch, "65748e97b43a5aadd4b34042881c80637c97a30b");
        assert!(metadata(dst.path().join(".git")).is_ok());
    }

    #[test]
    fn test_cloning_a_repo_with_a_specific_branch() {
        let dst = tmp_dir().unwrap();
        let repo_url = "https://github.com/cargo-generate/cargo-generate.git";

        let branch = RepoCloneBuilderGix::new(repo_url)
            .unwrap()
            .with_branch("feat/1037-gix-as-git2-sucessort-latest".into())
            .checkout(dst.path())
            .unwrap();

        let real_branch = get_repo_branch_at_path(dst.path()).unwrap();

        assert_eq!(real_branch, "feat/1037-gix-as-git2-sucessort-latest");
        assert_eq!(branch, real_branch);
        assert!(metadata(dst.path().join(".git")).is_ok());
    }

    #[test]
    fn test_new_clone_repo_builder() {
        assert!(
            RepoCloneBuilderGix::new("https://github.com/cargo-generate/cargo-generate.git")
                .is_ok()
        );
        assert!(
            RepoCloneBuilderGix::new("git@github.com:cargo-generate/cargo-generate.git").is_ok()
        );
        assert!(
            RepoCloneBuilderGix::new("/Users/I563162/workspace/cargo-generate/cargo-generate")
                .is_ok()
        );
    }

    // helper function to get the current branch name
    fn get_repo_branch_at_path(repo: &Path) -> Result<String> {
        // dbg output a directory listing
        eprintln!("repo contents at: {:?}", repo);
        let dir = std::fs::read_dir(repo).expect("a directory did not exist!!");
        for entry in dir {
            let entry = entry?;
            eprintln!(" - entry: {:?}", entry.path());
        }

        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(repo)
            .output()?;

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(branch)
    }
}
