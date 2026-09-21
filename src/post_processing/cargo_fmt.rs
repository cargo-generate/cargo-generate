//! The built-in formatting step that runs on the generated project.
//!
//! Templates rarely produce rustfmt-clean output, so formatting is a courtesy
//! `cargo-generate` performs itself. Two things make it safe to have on by
//! default:
//!
//! * It runs against the *final destination* rather than the temp template dir
//!   that hooks see — <https://github.com/cargo-generate/cargo-generate/issues/1435>
//!   is what that temp dir costs on Windows.
//! * It formats only the files this generation wrote. `--init` can expand a
//!   template into a subtree of somebody's existing project — adding a
//!   controller or a db module — and their untouched code must not be
//!   reformatted as a side effect.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use console::style;
use log::{info, warn};

use crate::config::Config;
use crate::emoji;

/// Cargo's own default when a manifest states no edition.
const DEFAULT_EDITION: &str = "2015";

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

/// Format the Rust files among `generated_files`, warning instead of failing.
///
/// A courtesy must not be able to fail someone's generation: a missing
/// `rustfmt`, a missing component and a rustfmt error all warn and return.
///
/// Files outside a cargo project are left alone — without a manifest there is
/// no edition to format them against, and nothing asked for them to be Rust.
pub fn format_generated_files(generated_files: &[PathBuf]) {
    // rustfmt takes one edition per invocation, so group by it. A generated
    // workspace can hold members on different editions.
    let mut by_edition: BTreeMap<String, Vec<&Path>> = BTreeMap::new();
    for file in generated_files
        .iter()
        .filter(|file| file.extension() == Some(OsStr::new("rs")))
    {
        if let Some(edition) = resolve_edition(file) {
            by_edition.entry(edition).or_default().push(file);
        }
    }

    if by_edition.is_empty() {
        return;
    }

    info!(
        "{} {}",
        emoji::WRENCH,
        style("Formatting the generated files").bold()
    );

    for (edition, files) in by_edition {
        run_rustfmt(&edition, &files);
    }
}

/// `rustfmt` rather than `cargo fmt`, because only `rustfmt` takes a file list.
///
/// `cargo fmt` formats whole packages, which is the wrong granularity twice
/// over: it would reformat a user's pre-existing code under `--init`, and with
/// `--all` it would reach into an enclosing workspace the generated project has
/// just been added to as a member.
///
/// `skip_children` is what makes the file list mean what it says. Given a crate
/// root, rustfmt otherwise walks its `mod` declarations and formats the whole
/// module tree — which under `--init` reaches files we never wrote. Every file
/// we *did* write is in this list already, so there is nothing to recurse for.
fn run_rustfmt(edition: &str, files: &[&Path]) {
    let result = Command::new("rustfmt")
        .arg("--edition")
        .arg(edition)
        .args(["--config", "skip_children=true"])
        .args(files)
        .output();

    match result {
        Ok(output) if output.status.success() => (),
        Ok(output) => warn_fmt_failed(&String::from_utf8_lossy(&output.stderr)),
        Err(e) => warn_fmt_failed(&e.to_string()),
    }
}

/// `log_formatter` already prefixes warnings with the warning emoji.
fn warn_fmt_failed(reason: &str) {
    let reason = reason.trim();
    let detail = if reason.is_empty() {
        String::from("rustfmt reported no reason")
    } else {
        reason.to_owned()
    };

    warn!(
        "{}",
        style(format!("Could not format the generated files: {detail}")).yellow()
    );
}

/// The edition `file` should be formatted against, or `None` when it belongs to
/// no cargo project at all.
///
/// Walks up from the file the way cargo does, so a file generated into a
/// workspace member resolves against that member rather than the workspace root.
fn resolve_edition(file: &Path) -> Option<String> {
    let mut inherited = false;

    for dir in file.ancestors().skip(1) {
        let Ok(manifest) = fs::read_to_string(dir.join("Cargo.toml")) else {
            continue;
        };
        // `toml::from_str`, not `str::parse`: `FromStr for Value` parses a
        // single TOML value, which a manifest is not.
        let Ok(manifest) = toml::from_str::<toml::Table>(&manifest) else {
            continue;
        };

        // `[workspace.package] edition` is what `edition.workspace = true`
        // below us refers to, and also the only edition a virtual manifest has.
        let workspace_edition = manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("package"))
            .and_then(|package| package.get("edition"))
            .and_then(toml::Value::as_str);

        if inherited {
            if let Some(edition) = workspace_edition {
                return Some(edition.to_owned());
            }
            continue;
        }

        match manifest
            .get("package")
            .map(|package| package.get("edition"))
        {
            // a package that states its edition outright
            Some(Some(toml::Value::String(edition))) => return Some(edition.clone()),
            // `edition.workspace = true` — keep walking up for the root
            Some(Some(_)) => inherited = true,
            // a package with no edition at all is a 2015 package
            Some(None) => return Some(String::from(DEFAULT_EDITION)),
            // a virtual manifest: its members were handled by their own
            // manifests, so only its workspace-wide edition can apply here
            None => {
                if let Some(edition) = workspace_edition {
                    return Some(edition.to_owned());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tmp_dir;

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

    /// `dir/Cargo.toml` with `manifest`, and a `src/main.rs` beneath it.
    fn manifest_with_source(dir: &Path, manifest: &str) -> PathBuf {
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("Cargo.toml"), manifest).unwrap();
        let source = dir.join("src").join("main.rs");
        fs::write(&source, "fn main(){}\n").unwrap();
        source
    }

    #[test]
    fn edition_comes_from_the_package() {
        let tmp = tmp_dir().unwrap();
        let source = manifest_with_source(
            tmp.path(),
            "[package]\nname = \"p\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );

        assert_eq!(resolve_edition(&source).as_deref(), Some("2021"));
    }

    #[test]
    fn a_package_without_an_edition_is_a_2015_package() {
        let tmp = tmp_dir().unwrap();
        let source =
            manifest_with_source(tmp.path(), "[package]\nname = \"p\"\nversion = \"0.1.0\"\n");

        assert_eq!(resolve_edition(&source).as_deref(), Some(DEFAULT_EDITION));
    }

    #[test]
    fn a_workspace_inherited_edition_resolves_at_the_root() {
        let tmp = tmp_dir().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\n\n[workspace.package]\nedition = \"2024\"\n",
        )
        .unwrap();
        let source = manifest_with_source(
            &tmp.path().join("member"),
            "[package]\nname = \"m\"\nversion = \"0.1.0\"\nedition.workspace = true\n",
        );

        assert_eq!(resolve_edition(&source).as_deref(), Some("2024"));
    }

    #[test]
    fn a_member_edition_wins_over_the_workspace_one() {
        let tmp = tmp_dir().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\n\n[workspace.package]\nedition = \"2015\"\n",
        )
        .unwrap();
        let source = manifest_with_source(
            &tmp.path().join("member"),
            "[package]\nname = \"m\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );

        assert_eq!(resolve_edition(&source).as_deref(), Some("2021"));
    }

    #[test]
    fn a_file_outside_any_cargo_project_has_no_edition() {
        let tmp = tmp_dir().unwrap();
        let source = tmp.path().join("loose.rs");
        fs::write(&source, "fn main(){}\n").unwrap();

        assert_eq!(resolve_edition(&source), None);
    }

    #[test]
    fn formatting_nothing_is_a_silent_no_op() {
        let tmp = tmp_dir().unwrap();
        let readme = tmp.path().join("README.md");
        fs::write(&readme, "# not rust\n").unwrap();

        // no panic, no error, nothing written
        format_generated_files(std::slice::from_ref(&readme));
        assert_eq!(fs::read_to_string(&readme).unwrap(), "# not rust\n");
    }

    #[test]
    fn a_rust_file_outside_a_cargo_project_is_left_alone() {
        let tmp = tmp_dir().unwrap();
        let source = tmp.path().join("loose.rs");
        fs::write(&source, "fn main(){}\n").unwrap();

        format_generated_files(std::slice::from_ref(&source));

        assert_eq!(fs::read_to_string(&source).unwrap(), "fn main(){}\n");
    }

    #[test]
    fn it_does_not_follow_mod_declarations_out_of_the_file_list() {
        let tmp = tmp_dir().unwrap();
        let root = manifest_with_source(
            tmp.path(),
            "[package]\nname = \"p\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        fs::write(&root, "mod theirs;\nfn main(){}\n").unwrap();
        let child = tmp.path().join("src").join("theirs.rs");
        fs::write(&child, "pub fn theirs(){}\n").unwrap();

        format_generated_files(std::slice::from_ref(&root));

        assert_eq!(
            fs::read_to_string(&root).unwrap(),
            "mod theirs;\nfn main() {}\n"
        );
        assert_eq!(
            fs::read_to_string(&child).unwrap(),
            "pub fn theirs(){}\n",
            "a `mod` child we did not generate must not be reformatted"
        );
    }

    #[test]
    fn it_formats_only_the_files_it_is_given() {
        let tmp = tmp_dir().unwrap();
        let generated = manifest_with_source(
            tmp.path(),
            "[package]\nname = \"p\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        let untouched = tmp.path().join("src").join("theirs.rs");
        fs::write(&untouched, "pub fn theirs(){}\n").unwrap();

        format_generated_files(std::slice::from_ref(&generated));

        assert_eq!(fs::read_to_string(&generated).unwrap(), "fn main() {}\n");
        assert_eq!(
            fs::read_to_string(&untouched).unwrap(),
            "pub fn theirs(){}\n",
            "a file we did not generate must not be reformatted"
        );
    }
}
