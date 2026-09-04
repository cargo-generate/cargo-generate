//! A scaffolder that ships its own blueprint.
//!
//! cargo-generate is used here purely as a template engine. The
//! blueprint is a directory in this crate, so there is nothing to
//! clone and no reason to carry a git implementation — see the
//! dependency line in `Cargo.toml`.

// ANCHOR: imports
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};
// ANCHOR_END: imports

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // ANCHOR: build_args
    // The blueprint lives next to this source file. `CARGO_MANIFEST_DIR`
    // is resolved at compile time, so the path holds wherever the
    // checkout sits.
    let args = GenerateArgs {
        name: Some("my-service".to_string()),
        // Without the `git` feature this is already the default, but
        // saying it is what makes the example independent of that.
        vcs: Some(Vcs::None),
        template_path: TemplatePath {
            path: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/blueprint").to_string()),
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
