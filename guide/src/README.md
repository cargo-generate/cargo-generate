# Introduction

> cargo, make me a project

[![Build status](https://github.com/cargo-generate/cargo-generate/workflows/Build/badge.svg)](https://github.com/cargo-generate/cargo-generate/actions?query=workflow%3ABuild+branch%3Amain+)
[![crates.io](https://img.shields.io/crates/v/cargo-generate.svg)](https://crates.io/crates/cargo-generate)
[![dependency status](https://deps.rs/repo/github/cargo-generate/cargo-generate/status.svg)](https://deps.rs/repo/github/cargo-generate/cargo-generate)

`cargo-generate` is a developer tool to help you get up and running quickly with a new Rust
project by leveraging a pre-existing git repository as a template.

cargo-generate uses [Shopify's Liquid] template language,
[Rhai](https://docs.rs/rhai/latest/rhai/) for hook scripts and [regex](https://docs.rs/regex/latest/regex/) for placeholders.

Due to the use of [Shopify's Liquid], `cargo-generate` special cases files with the file-ending
`.liquid`, by simply removing the file-ending when processing the files. If you, as a template 
author, truly want the `.liquid` file-ending, you need to repeat it twice!

An Example: the file `main.rs.liquid` will be renamed after templating to `main.rs`

Here's an example of using `cargo-generate` with [this template]:

![demo.gif](./demo.gif)

[this template]: https://github.com/rustwasm/wasm-pack-template
[Shopify's Liquid]: http://liquidmarkup.org/
