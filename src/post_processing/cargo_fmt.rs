//! The built-in `cargo fmt` step that runs on the generated project.
//!
//! Templates rarely produce rustfmt-clean output, so formatting is a courtesy
//! `cargo-generate` performs itself. It runs against the *final destination*
//! rather than the temp template dir that hooks see — see
//! <https://github.com/cargo-generate/cargo-generate/issues/1435> for what
//! that temp dir costs on Windows.

use std::path::Path;
use std::process::Command;

use console::style;
use log::{info, warn};

use crate::config::Config;
use crate::emoji;

/// Whether the generated project should be formatted.
///
/// `--no-fmt` wins over everything; an explicit `cargo-fmt` post hook wins over
/// a template opting out; otherwise `[template] fmt` decides, defaulting to on.
pub fn should_format(config: &Config, no_fmt: bool) -> bool {
    if no_fmt {
        return false;
    }
    if config.has_cargo_fmt_hook() {
        return true;
    }
    config
        .template
        .as_ref()
        .and_then(|template| template.fmt)
        .unwrap_or(true)
}

/// A project we can hand to `cargo fmt` — one with a manifest.
///
/// Templates that are not cargo projects have nothing to format, and this gate
/// is what makes formatting-by-default safe for them.
fn is_cargo_project(project_dir: &Path) -> bool {
    project_dir.join("Cargo.toml").is_file()
}

/// Format `project_dir` with `cargo fmt`, warning instead of failing.
///
/// A courtesy must not be able to fail someone's generation: a missing `cargo`,
/// a missing `rustfmt` component and a rustfmt error all warn and return.
pub fn format_project(project_dir: &Path) {
    if !is_cargo_project(project_dir) {
        return;
    }

    info!(
        "{} {}",
        emoji::WRENCH,
        style("Formatting the generated project").bold()
    );

    // Plain `cargo fmt`, never `--all`: when the project has just been added as
    // a member of an enclosing workspace, `--all` resolves to that outer
    // workspace root and would reformat the user's unrelated crates.
    let result = Command::new("cargo")
        .arg("fmt")
        .current_dir(project_dir)
        .output();

    match result {
        Ok(output) if output.status.success() => (),
        Ok(output) => warn_fmt_failed(&String::from_utf8_lossy(&output.stderr)),
        Err(e) => warn_fmt_failed(&e.to_string()),
    }
}

/// `log_formatter` already prefixes warnings with the warning emoji.
fn warn_fmt_failed(reason: &str) {
    warn!(
        "{}",
        style(format!(
            "Could not format the generated project: {}",
            reason.trim()
        ))
        .yellow()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tmp_dir;
    use std::fs;

    fn config_from(toml: &str) -> Config {
        Config::try_from(toml.to_string()).unwrap()
    }

    #[test]
    fn formats_by_default() {
        assert!(should_format(&config_from(""), false));
    }

    #[test]
    fn no_fmt_flag_wins_over_everything() {
        let config = config_from(
            r#"
            [hooks]
            post = ["cargo-fmt"]
            "#,
        );

        assert!(!should_format(&config, true));
    }

    #[test]
    fn template_can_opt_out() {
        let config = config_from(
            r#"
            [template]
            fmt = false
            "#,
        );

        assert!(!should_format(&config, false));
    }

    #[test]
    fn named_hook_overrides_template_opt_out() {
        let config = config_from(
            r#"
            [template]
            fmt = false

            [hooks]
            post = ["cargo-fmt"]
            "#,
        );

        assert!(should_format(&config, false));
    }

    #[test]
    fn a_directory_without_a_manifest_is_not_a_cargo_project() {
        let dir = tmp_dir().unwrap();

        assert!(!is_cargo_project(dir.path()));
        // and formatting it is a silent no-op, not an error
        format_project(dir.path());
    }

    #[test]
    fn a_directory_with_a_manifest_is_a_cargo_project() {
        let dir = tmp_dir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();

        assert!(is_cargo_project(dir.path()));
    }
}
