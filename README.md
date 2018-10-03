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

```
cargo install cargo-generate
```

`cargo-generate` has a few dependencies that need to be available before it can be installed and used.

* `openssl`: See the [`openssl-sys` crate readme] on how to obtain the openssl library for your system.
* [`git`]: It is used to download the templates.
* `cmake`: Check if it is installed by typing `cmake --version` in a terminal or command line window. If it is not available, check your package
  manager or see the [cmake homepage].

[`openssl-sys` crate readme]: https://crates.io/crates/openssl-sys
[`git`]: https://git-scm.com/downloads
[cmake homepage]: https://cmake.org/download/

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

You can also add a `.genignore` file to your template. The files listed in the `.genignore` file
will be removed from the local machine when `cargo-generate` is run on the end user's machine.

Here's a list of [currently available templates](TEMPLATES.md).
If you have a great template that you'd like to feature here, please [file an issue or a PR]!

[file an issue or a PR]: https://github.com/ashleygwilliams/cargo-generate/issues

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
