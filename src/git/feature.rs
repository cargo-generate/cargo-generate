//! What "built without the `git` feature" means, and what to do
//! about it.
//!
//! Only a library consumer can reach any of this — the CLI always
//! ships with default features — so the wording addresses them: it
//! names the API member that was supplied and the Cargo.toml line
//! that fixes it, not command-line flags.
//!
//! Always compiled, so the wording has exactly one definition and
//! call sites need no `#[cfg]` of their own.

/// Error returned when git-requiring input reaches a build compiled
/// without the `git` feature.
///
/// `what` names the offending input as the caller expressed it —
/// `"TemplatePath::git"`, `"Vcs::Git"`, or a phrase for input that
/// came from a config file — not the internal function that happened
/// to notice.
#[cfg(not(feature = "git"))]
pub fn feature_disabled(what: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "{what} needs git support, but this build of cargo-generate was compiled \
         without the `git` cargo feature.\n\
         \n\
         Enable it in your Cargo.toml:\n\
         \x20   cargo-generate = {{ version = \"…\", features = [\"git\"] }}\n\
         \n\
         Templates from a local path work without it."
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
        "initializing a git repository in the generated project \
         (`Vcs::Git`, or `vcs` in the template's or a favorite's config)",
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
        assert!(ensure_available("`Vcs::Git`").is_ok());
    }

    #[cfg(not(feature = "git"))]
    #[test]
    fn ensure_available_errors_without_the_feature() {
        let err = ensure_available("`Vcs::Git`").unwrap_err().to_string();
        assert!(
            err.contains("`Vcs::Git`"),
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
