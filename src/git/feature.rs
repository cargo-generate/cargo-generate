//! What "built without the `git` feature" means to a user.
//!
//! Always compiled, so the wording has exactly one definition and
//! call sites need no `#[cfg]` of their own.

/// Error returned when git-requiring input reaches a build compiled
/// without the `git` feature.
///
/// `what` names the offending input from the *user's* point of view
/// — `"--git <url>"`, `"--vcs git"` — not the internal function that
/// happened to notice.
#[cfg(not(feature = "git"))]
pub fn feature_disabled(what: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "{what} requires git support, but this build of cargo-generate was compiled \
         without the `git` cargo feature.\n\
         \n\
         To enable it:\n\
         \x20 * as a binary:  cargo install cargo-generate          (default features include `git`)\n\
         \x20 * as a library: cargo-generate = {{ version = \"…\", features = [\"git\"] }}\n\
         \n\
         Templates from a local path (`--path`) work without the feature."
    )
}

/// `Ok(())` when the `git` feature is enabled; the
/// `feature_disabled` error for `what` otherwise.
///
/// Used where the *existing* API accepts git-related input that the
/// type system cannot reject — most importantly `Vcs::Git`, which is
/// part of the invariant-shape API and therefore exists in every
/// build.
///
/// # Errors
///
/// Returns an error when the `git` feature is disabled.
#[cfg(feature = "git")]
#[allow(clippy::missing_const_for_fn)]
pub fn ensure_available(_what: &str) -> anyhow::Result<()> {
    Ok(())
}

/// See the `cfg(feature = "git")` twin above.
///
/// # Errors
///
/// Always errors: this build has no git support.
#[cfg(not(feature = "git"))]
pub fn ensure_available(what: &str) -> anyhow::Result<()> {
    Err(feature_disabled(what))
}

/// Stand-in for `gix::try_get_branch_from_path` without the feature.
///
/// Always `None`: nothing consumes an inferred branch in a build
/// without git support. The only reader is `git::init`, and reaching
/// it at all means an explicit git request already failed.
#[cfg(not(feature = "git"))]
pub const fn try_get_branch_from_path(_path: &std::path::Path) -> Option<String> {
    None
}

/// Stand-in for `init::init` without the feature.
///
/// `Vcs::Git` is existing API and exists in every build, so
/// `Vcs::initialize` has a `Git` arm in every build and must compile
/// against *something*. Cloning needs no equivalent: `Source::Git`
/// does not exist without the feature, so nothing reaches the clone
/// path.
///
/// This is a safety net, not the primary diagnostic — input
/// validation bails earlier and with a better-scoped message. It
/// catches what bypasses that, most notably a template whose
/// `cargo-generate.toml` asks for `vcs = "Git"`.
///
/// # Errors
///
/// Always errors: this build has no git support.
#[cfg(not(feature = "git"))]
pub fn init(
    _project_dir: &std::path::Path,
    _branch: Option<&str>,
    _force: bool,
) -> anyhow::Result<()> {
    Err(feature_disabled(
        "initializing a git repository for the generated project (`--vcs git`)",
    ))
}

/// Stand-in for `gitconfig::read_config_string` without the feature.
///
/// Reading `user.name` / `user.email` out of git config is a *fallback*
/// in a chain that starts with environment variables — nobody asked for
/// git, so this degrades to `None` rather than bailing, exactly like the
/// default VCS degrades to `Vcs::None`.
#[cfg(not(feature = "git"))]
#[allow(clippy::missing_const_for_fn)]
pub fn read_config_string(_key: &str, _cwd: &std::path::Path) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "git")]
    #[test]
    fn ensure_available_is_ok_with_the_feature() {
        assert!(ensure_available("`--vcs git`").is_ok());
    }

    #[cfg(not(feature = "git"))]
    #[test]
    fn ensure_available_errors_without_the_feature() {
        let err = ensure_available("`--vcs git`").unwrap_err().to_string();
        assert!(
            err.contains("`--vcs git`"),
            "names the offending input: {err}"
        );
        assert!(
            err.contains("`git` cargo feature"),
            "names the feature: {err}"
        );
    }

    #[cfg(not(feature = "git"))]
    #[test]
    fn read_config_string_is_none_without_the_feature() {
        assert_eq!(
            read_config_string("user.name", std::path::Path::new(".")),
            None
        );
    }

    #[cfg(not(feature = "git"))]
    #[test]
    fn init_stub_reports_the_disabled_feature() {
        let err = init(std::path::Path::new("."), None, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("`git` cargo feature"), "{err}");
    }
}
