# Library Usage

`cargo-generate` is also a Rust library. Any binary or library crate can depend
on it and drive template generation programmatically — the same code path the
`cargo generate` CLI uses under the hood.

The canonical worked example lives in the repository at
[`examples/how-to-use-cargo-gen-as-library/`][ex]. This chapter walks through
its pieces, pulling the snippets straight from that file so they can never drift
apart.

## Depend on `cargo-generate`

Add the crate to your `Cargo.toml`. Turning `default-features` off keeps your
build free of the CLI-only feature gates:

```toml
[dependencies]
cargo-generate = { version = "*", default-features = false }
```

## Imports

Three types cover the common cases:

```rust
{{#include ../../examples/how-to-use-cargo-gen-as-library/src/main.rs:imports}}
```

* [`GenerateArgs`] mirrors the top-level CLI arguments.
* [`TemplatePath`] describes where the template comes from — a git url, a local
  path, a favorite, etc.
* [`Vcs`] controls what version control system (if any) is initialized in the
  generated project.

## 1. Build a `GenerateArgs`

Populate only the fields you care about; everything else falls back to
`GenerateArgs::default()` / `TemplatePath::default()`:

```rust
{{#include ../../examples/how-to-use-cargo-gen-as-library/src/main.rs:build_args}}
```

The example above is equivalent to running the CLI as:

```sh
cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name my-project
```

## 2. Call `generate`

`generate` runs the same flow the CLI would — clone the template, expand
placeholders, honor hooks, and (if a VCS is requested) initialize the new
repository. On success it returns the [`PathBuf`] of the generated project:

```rust
{{#include ../../examples/how-to-use-cargo-gen-as-library/src/main.rs:call}}
```

## Running the example

From a checkout of this repository:

```sh
cd examples/how-to-use-cargo-gen-as-library
cargo run
```

You will end up with a freshly generated `my-project/` in the current
directory, exactly as if you had invoked `cargo generate` yourself.

[ex]: https://github.com/cargo-generate/cargo-generate/tree/main/examples/how-to-use-cargo-gen-as-library
[`GenerateArgs`]: https://docs.rs/cargo-generate/latest/cargo_generate/struct.GenerateArgs.html
[`TemplatePath`]: https://docs.rs/cargo-generate/latest/cargo_generate/struct.TemplatePath.html
[`Vcs`]: https://docs.rs/cargo-generate/latest/cargo_generate/enum.Vcs.html
[`PathBuf`]: https://doc.rust-lang.org/std/path/struct.PathBuf.html
