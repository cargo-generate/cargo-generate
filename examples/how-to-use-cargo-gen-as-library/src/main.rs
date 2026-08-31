// ANCHOR: imports
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};
// ANCHOR_END: imports

fn main() {
    // We use wasm-pack as in the README example,
    // So this is equivalent to run cargo-generate like:
    // ```sh
    // cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name my-project
    // ```
    // ANCHOR: build_args
    let wasm_pack_args = GenerateArgs {
        name: Some("my-project".to_string()),
        vcs: Some(Vcs::Git),
        template_path: TemplatePath {
            git: Some("https://github.com/rustwasm/wasm-pack-template.git".to_string()),
            ..TemplatePath::default()
        },
        ..GenerateArgs::default()
    };
    // ANCHOR_END: build_args

    // ANCHOR: call
    let _path = generate(wasm_pack_args).expect("something went wrong!");
    // ANCHOR_END: call
}
