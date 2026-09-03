//! Everything that touches git, behind one facade.
//!
//! The `git` cargo feature decides what this module can *do*, never
//! what it looks like: exported names and signatures are the same in
//! both configurations, so callers carry no `#[cfg]`.
//!
//! Always compiled — fs-only, no `gix`, and on the plain-copy path
//! that works without the feature:
//! [`history`], [`tmp`], [`feature`].
//!
//! Feature-gated — the git runtime and its `gix` dependency:
//! [`gitconfig`], [`gix`], [`init`], [`utils`], and branch detection.
//!
//! [`feature`] holds the "this build has no git" error and the
//! stand-ins the always-compiled callers need: [`init`], because
//! `Vcs::Git` is public API and exists in every build, plus
//! `read_config_string` and `try_get_branch_from_path`, which return
//! `None` — author lookup falls through to its environment-variable
//! chain, and nothing reads an inferred branch when there is no git
//! to init. Cloning needs no stand-in: without the feature there is
//! no git source to reach it.

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
