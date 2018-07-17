# cargo-generate

> cargo, make me a project

[![Build status](https://ci.appveyor.com/api/projects/status/t3f0wtt99u0p20p4/branch/master?svg=true)](https://ci.appveyor.com/project/ashleygwilliams/cargo-generate/branch/master)
[![Build Status](https://travis-ci.com/ashleygwilliams/cargo-generate.svg?branch=master)](https://travis-ci.com/ashleygwilliams/cargo-generate)

`cargo-generate` is a developer tool to help you get up and running quickly with a new Rust
project by leveraging a pre-existing git repository as a template. 

![demo.gif](./demo.gif)

## Installation

```
cargo install cargo-generate
```

## Usage

Standard usage is to pass a `--git` flag to `cargo generate`. This will prompt you to enter the name of your project.

```
cargo generate --git https://github.com/githubusername/mytemplate.git
```

You can also pass the name of your project to the tool using the `--name` flag:

```
cargo generate --git https://github.com/githubusername/mytemplate.git --name myproject
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
