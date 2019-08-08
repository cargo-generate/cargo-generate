# cargo-generate

> cargo, make me a project

[![Build status](https://ci.appveyor.com/api/projects/status/t3f0wtt99u0p20p4/branch/master?svg=true)](https://ci.appveyor.com/project/ashleygwilliams/cargo-generate/branch/master)
[![Build Status](https://travis-ci.com/ashleygwilliams/cargo-generate.svg?branch=master)](https://travis-ci.com/ashleygwilliams/cargo-generate)

`cargo-generate` is a developer tool to help you get up and running quickly with a new Rust
project by leveraging a pre-existing git repository as a template.

Here's an example of using `cargo-generate` with [this template]:
![demo.gif](./demo.gif)

[this template]: https://github.com/ashleygwilliams/wasm-pack-template

## Installation

### Using `cargo` with system's OpenSSL

```
cargo install cargo-generate
```

See the [`openssl-sys` crate readme] on how to obtain the OpenSSL library for your system. Alternatively, use the `vendored-openssl` flag if you do not want to install OpenSSL.

[`openssl-sys` crate readme]: https://crates.io/crates/openssl-sys

### Using `cargo` with vendored OpenSSL

```
cargo install cargo-generate --features vendored-openssl
```

### Manual Install:

1. Download the binary tarball for your platform from our [releases page](https://github.com/ashleygwilliams/cargo-generate/releases).

2. Unpack the tarball and place the binary `cargo-generate` in `~/.cargo/bin/`

## Usage

Standard usage is to pass a `--git` flag to `cargo generate` or short `cargo gen`. This will prompt you to enter the name of your project.

```
cargo generate --git https://github.com/githubusername/mytemplate.git
```

You can also pass the name of your project to the tool using the `--name` or `-n` flag:

```
cargo generate --git https://github.com/githubusername/mytemplate.git --name myproject
```

## Templates

Templates are git repositories whose files contain placeholders. The current
supported placeholders are:

- `{{authors}}`: this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration.
- `{{project-name}}`: this is supplied by either passing the `--name` flag to the command or working with the interactive CLI to supply a name.
- `{{crate_name}}`: the snake_case_version of `project-name`
- dates: the liquid date filter is enabled for this project. This means you can write something like `{{ "now" | date: "%Y-%m-%d %H:%M" }}`. For more information, check out the [Liquid Documentation on `date`].

[Liquid Documentation on `date`]: https://shopify.github.io/liquid/filters/date/

You can also add a `.genignore` file to your template. The files listed in the `.genignore` file
will be removed from the local machine when `cargo-generate` is run on the end user's machine.
The `.genignore` file is always ignored, so there is no need to list it in the `.genignore` file.

Here's a list of [currently available templates](TEMPLATES.md).
If you have a great template that you'd like to feature here, please [file an issue or a PR]!

[file an issue or a PR]: https://github.com/ashleygwilliams/cargo-generate/issues

## Include / Exclude

Templates support a `cargo-generate.toml`, with a "template" section that allows you to configure the files that will be processed by `cargo-generate`.
The behavior mirrors Cargo's Include / Exclude functionality, which is [documented here](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields-optional)

```toml
[template]
include = ["Cargo.toml"]
# include and exclude are exclusive, if both appear we will use include
exclude = ["*.c"]
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.  
If you want to contribute to `cargo-generate`, please read our [CONTRIBUTING notes].

[CONTRIBUTING notes]: CONTRIBUTING.md
