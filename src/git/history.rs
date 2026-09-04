//! Strip a materialized template's `.git` directory.
//!
//! Always compiled: every fetch path calls this, with or without the
//! `git` feature. Uses the `remove_dir_all` crate rather than
//! `std::fs` — git repos on Windows carry read-only pack files that
//! trip the std version, which is why cargo uses it too — and retries
//! briefly when a just-cloned `.git` is still locked.

use std::io;
use std::path::Path;
use std::{thread::sleep, time::Duration};

use remove_dir_all::remove_dir_all;

/// Remove the `.git` directory under `project_dir`, if any.
///
/// * If `.git` does not exist, returns `Ok(())`.
/// * On Windows, retries up to 5 times with exponential backoff when
///   the error indicates the directory is still in use.
///
/// # Errors
///
/// Returns the underlying I/O error when `.git` exists but cannot be
/// removed.
pub fn remove_history(project_dir: &Path) -> io::Result<()> {
    let git_dir = project_dir.join(".git");
    if !git_dir.is_dir() {
        return Ok(());
    }

    let mut attempt = 0_u8;
    loop {
        attempt += 1;
        match remove_dir_all(&git_dir) {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempt >= 5 {
                    return Err(e);
                }
                if e.to_string().contains(
                    "The process cannot access the file because it is being used by another process.",
                ) {
                    let wait_for = Duration::from_secs(2_u64.pow(u32::from(attempt - 1)));
                    log::warn!(
                        "`.git` cleanup failed with a Windows process-lock error. \
                         Retry in {wait_for:?}"
                    );
                    sleep(wait_for);
                } else {
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn removes_dot_git_when_present() {
        let dir = TempDir::new().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        remove_history(dir.path()).unwrap();

        assert!(!git_dir.exists());
    }

    #[test]
    fn is_noop_when_no_dot_git() {
        let dir = TempDir::new().unwrap();
        remove_history(dir.path()).unwrap();
    }

    #[test]
    fn is_noop_when_dot_git_is_a_file_not_a_dir() {
        // Git submodules use a `.git` *file* pointing at the parent
        // repo's git dir. Historical behavior: leave it alone.
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".git"), "gitdir: ../.git/modules/x").unwrap();
        remove_history(dir.path()).unwrap();
        assert!(dir.path().join(".git").exists());
    }
}
