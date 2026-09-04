//! A scaffolder that carries its own blueprint.
//!
//! cargo-generate is used here purely as a template engine. The
//! blueprint is compiled into this binary, so there is nothing to
//! clone and no reason to carry a git implementation — see the
//! dependency line in `Cargo.toml`.

// ANCHOR: imports
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};
use include_dir::{include_dir, Dir};
// ANCHOR_END: imports

use std::error::Error;
use tempfile::TempDir;

// ANCHOR: embed
/// The blueprint, as bytes in the binary. Nothing is read from the
/// source tree at runtime, so the tool keeps working after
/// `cargo install`.
static BLUEPRINT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/blueprint");
// ANCHOR_END: embed

fn main() -> Result<(), Box<dyn Error>> {
    // ANCHOR: unpack
    // cargo-generate reads templates from a directory, so hand it one.
    // `tmp` has to outlive the call — dropping it deletes the blueprint.
    let tmp = TempDir::new()?;
    BLUEPRINT.extract(tmp.path())?;
    // ANCHOR_END: unpack

    // ANCHOR: build_args
    let args = GenerateArgs {
        name: Some("my-service".to_string()),
        // Without the `git` feature this is already the default, but
        // saying it is what makes the example independent of that.
        vcs: Some(Vcs::None),
        template_path: TemplatePath {
            path: Some(tmp.path().display().to_string()),
            ..TemplatePath::default()
        },
        ..GenerateArgs::default()
    };
    // ANCHOR_END: build_args

    // ANCHOR: call
    let path = generate(args)?;
    // ANCHOR_END: call

    println!("generated {}", path.display());
    Ok(())
}
