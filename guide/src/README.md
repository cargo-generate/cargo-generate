# Introduction

```rust
cargo generate /me/a-project
```

[![Build status](https://github.com/cargo-generate/cargo-generate/workflows/Build/badge.svg)](https://github.com/cargo-generate/cargo-generate/actions?query=workflow%3ABuild+branch%3Amain+)
[![crates.io](https://img.shields.io/crates/v/cargo-generate.svg)](https://crates.io/crates/cargo-generate)
[![dependency status](https://deps.rs/repo/github/cargo-generate/cargo-generate/status.svg)](https://deps.rs/repo/github/cargo-generate/cargo-generate)

`cargo-generate` is a Rust developer tool to quickly setup any Rust project,
by automating repetitive or intricate tasks.

Project templates are ***any*** files.  Alternatively, files can leverage
[Shopify's Liquid](http://liquidmarkup.org/) templates and Rust's 
[regex crate](https://docs.rs/regex/latest/regex/) to add crates,
features, initializers, etc. to your freshly created Rust project or an 
existing Rust project.

Here's an example of using `cargo-generate` with [this template]:
![demo.gif](./demo.gif)

[this template]: https://github.com/ashleygwilliams/wasm-pack-template
