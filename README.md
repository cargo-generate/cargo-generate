# cargo-generate

> cargo, make me a project

[![Build status](https://github.com/cargo-generate/cargo-generate/workflows/Build/badge.svg)](https://github.com/cargo-generate/cargo-generate/actions?query=workflow%3ABuild+branch%3Amain+)
[![crates.io](https://img.shields.io/crates/v/cargo-generate.svg)](https://crates.io/crates/cargo-generate)
[![dependency status](https://deps.rs/repo/github/cargo-generate/cargo-generate/status.svg)](https://deps.rs/repo/github/cargo-generate/cargo-generate)

`cargo-generate` is a developer tool to help you get up and running quickly with a new Rust
project by leveraging a pre-existing git repository as a template.

Here's an example of using `cargo-generate` with [this template]:
![demo.gif](./demo.gif)

[this template]: https://github.com/ashleygwilliams/wasm-pack-template

## Documentation

See the `cargo-generate` [guide](https://cargo-generate.github.io/cargo-generate/index.html) for complete documentation.

## Quickstart
### Installation

```sh
cargo install cargo-generate
```

### Usage

```sh
# templates on github
cargo generate --git https://github.com/username-on-github/mytemplate.git

# or just
cargo generate username-on-github/mytemplate

# templates on other git platforms
cargo generate gl:username-on-gitlab/mytemplate
cargo generate bb:username-on-bitbucket/mytemplate

# this scheme is also available for github
cargo generate gh:username-on-github/mytemplate

# for a complete list of arguments and options
cargo help generate
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
If you want to contribute to `cargo-generate`, please read our [CONTRIBUTING notes].

[CONTRIBUTING notes]: CONTRIBUTING.md
