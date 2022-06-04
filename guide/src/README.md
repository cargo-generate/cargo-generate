# Introduction

> cargo, make me a project

[![Build status](https://github.com/cargo-generate/cargo-generate/workflows/Build/badge.svg)](https://github.com/cargo-generate/cargo-generate/actions?query=workflow%3ABuild+branch%3Amain+)
[![crates.io](https://img.shields.io/crates/v/cargo-generate.svg)](https://crates.io/crates/cargo-generate)
[![dependency status](https://deps.rs/repo/github/cargo-generate/cargo-generate/status.svg)](https://deps.rs/repo/github/cargo-generate/cargo-generate)

`cargo-generate` is a developer tool to help you get up and running quickly with a new Rust
project by leveraging a pre-existing git repository as a template.

cargo-generate uses [Shopify's Liquid](http://liquidmarkup.org/) template language,
[Rhai](https://docs.rs/rhai/latest/rhai/) for hook scripts and [regex](https://docs.rs/regex/latest/regex/) for placeholders.

Here's an example of using `cargo-generate` with [this template]:
![demo.gif](./demo.gif)

[this template]: https://github.com/ashleygwilliams/wasm-pack-template
