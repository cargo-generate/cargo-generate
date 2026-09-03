//! Facade over git-touching operations.
//!
//! * `history`, `tmp`, `feature` are always compiled — fs-only, no
//!   `gix` needed, and called from the always-compiled `Local`
//!   template path.
//! * `gitconfig`, `init`, `utils` are the real runtime, compiled only
//!   with the feature, as is branch detection from `crate::gix`.
//! * `init` and `try_get_branch_from_path` additionally have stubs in
//!   `feature`: `Vcs::Git` is part of the invariant-shape existing API
//!   and exists in every build, and nothing reads an inferred branch
//!   when there is no git to init. `clone_git_template_into_temp`
//!   needs no stub — without the feature there is no `Source::Git` to
//!   reach it.

mod feature;
mod history;
mod tmp;

pub use feature::ensure_available;
pub use history::remove_history;
pub use tmp::tmp_dir;

#[cfg(not(feature = "git"))]
pub use feature::feature_disabled;
#[cfg(not(feature = "git"))]
pub use feature::init;
#[cfg(not(feature = "git"))]
pub use feature::read_config_string;
#[cfg(not(feature = "git"))]
pub use feature::try_get_branch_from_path;

#[cfg(feature = "git")]
pub mod gitconfig;
#[cfg(feature = "git")]
pub mod gix;
#[cfg(feature = "git")]
mod init;
#[cfg(feature = "git")]
pub mod utils;
#[cfg(feature = "git")]
pub use gitconfig::read_config_string;
#[cfg(feature = "git")]
pub use gix::try_get_branch_from_path;
#[cfg(feature = "git")]
pub use init::init;
#[cfg(feature = "git")]
pub use utils::clone_git_template_into_temp;
