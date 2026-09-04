# Library Usage

`cargo-generate` is also a Rust library. Any crate can depend on it and drive
template generation from code, using the same path as the `cargo generate` CLI.

A full worked example lives in this repository at
[`examples/how-to-use-cargo-gen-as-library/`][ex]. The snippets below are
pulled straight from that file with mdbook's `{{#include}}`, so they cannot
drift out of sync.

## Depend on `cargo-generate`

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
cargo-generate = "*"
```

### Cargo features

| Feature | Default | Description                                                                                                                    |
|---------|:-------:|----------------------------------------------------------------------------------------------------------------------------------|
| `git`   | ✔       | Fetching templates from git repositories, and initializing a repository in the generated project. Pulls in `gix` and `reqwest`. |

If your tool only ever generates from **local** templates, turning `git` off
drops the whole gix/reqwest/rustls tree:

```toml
[dependencies]
cargo-generate = { version = "*", default-features = false }
```

Your code does not change. [`GenerateArgs`], [`TemplatePath`] and [`Vcs`] keep
every field, so whatever compiled with default features still compiles. What
changes is at runtime: asking for a git template — as the example on this page
does — or for a git repository in the generated project returns an error
naming the feature rather than doing the work. Local-path templates, hooks,
rendering and workspace membership are unaffected.

## Imports

Three types cover the common cases:

```rust
{{#include ../../examples/how-to-use-cargo-gen-as-library/src/main.rs:imports}}
```

* [`GenerateArgs`] mirrors the top-level CLI arguments.
* [`TemplatePath`] describes where the template comes from: a git url, a local
  path, a favorite, etc.
* [`Vcs`] controls which version control system (if any) is initialized in the
  generated project.

## 1. Build a `GenerateArgs`

Populate only the fields you care about; everything else falls back to
`GenerateArgs::default()` / `TemplatePath::default()`:

```rust
{{#include ../../examples/how-to-use-cargo-gen-as-library/src/main.rs:build_args}}
```

The example above is equivalent to running:

```sh
cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name my-project
```

## 2. Call `generate`

`generate` runs the same flow as the CLI: clone the template, expand
placeholders, run hooks, and (if a VCS is requested) initialize the new
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

This creates a `my-project/` directory in the current folder, the same result
you would get from running `cargo generate` on the command line.

[ex]: https://github.com/cargo-generate/cargo-generate/tree/main/examples/how-to-use-cargo-gen-as-library
[`GenerateArgs`]: https://docs.rs/cargo-generate/latest/cargo_generate/struct.GenerateArgs.html
[`TemplatePath`]: https://docs.rs/cargo-generate/latest/cargo_generate/struct.TemplatePath.html
[`Vcs`]: https://docs.rs/cargo-generate/latest/cargo_generate/enum.Vcs.html
[`PathBuf`]: https://doc.rust-lang.org/std/path/struct.PathBuf.html
