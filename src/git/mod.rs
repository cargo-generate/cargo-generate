//! Facade over git-touching operations. Runtime files (`gitconfig`,
//! `utils`, `init`) are re-exported below, as is branch detection
//! from `crate::gix`. Task 4 swaps those for a
//! `#[cfg]`-selected pair (real vs `feature` stub); for now
//! everything stays unconditional.
//!
//! cargo-generate (as an application) wants from the git module:
//! 1. cloning a remote
//! 2. initializing a freshly generated template
//! 3. removing history from a cloned template
//!
//! Assumptions:
//! * `--git <url>` should only be parsed the same way `git clone <url>` would
//! * submodules are cloned by default, but can be skipped by `--skip-submodules`
//! * `.git` should be removed to make a clean repository
//! * if `<url>` is a local path on the system the clone should also be done the
//!   same way as `git clone` — there is `--path` for different behavior

pub mod gitconfig;
mod history;
mod init;
mod tmp;
pub mod utils;

pub use crate::gix::try_get_branch_from_path;
pub use history::remove_history;
pub use init::init;
pub use tmp::tmp_dir;
pub use utils::clone_git_template_into_temp;
