# cargo-generate

> cargo, make me a project

[![Build status](https://github.com/cargo-generate/cargo-generate/workflows/Build/badge.svg)](https://github.com/cargo-generate/cargo-generate/actions?query=workflow%3ABuild+branch%3Amaster+)
[![crates.io](https://img.shields.io/crates/v/cargo-generate.svg)](https://crates.io/crates/cargo-generate)
[![dependency status](https://deps.rs/repo/github/cargo-generate/cargo-generate/status.svg)](https://deps.rs/repo/github/cargo-generate/cargo-generate)

`cargo-generate` is a developer tool to help you get up and running quickly with a new Rust
project by leveraging a pre-existing git repository as a template.

Here's an example of using `cargo-generate` with [this template]:
![demo.gif](./demo.gif)

[this template]: https://github.com/ashleygwilliams/wasm-pack-template

## Installation

### Using `cargo` with system's OpenSSL

```sh
cargo install cargo-generate
```

See the [`openssl-sys` crate readme] on how to obtain the OpenSSL library for your system. Alternatively, use the `vendored-openssl` flag if you do not want to install OpenSSL.

[`openssl-sys` crate readme]: https://crates.io/crates/openssl-sys

### Using `cargo` with vendored OpenSSL

> NOTE: `vendored-openssl` requires the following packages to be installed:
>
> -   libssl-dev
> -   gcc
> -   m4
> -   ca-certificates
> -   make
> -   perl

```sh
cargo install cargo-generate --features vendored-openssl
```

### Manual Install:

1. Download the binary tarball for your platform from our [releases page](https://github.com/cargo-generate/cargo-generate/releases).

2. Unpack the tarball and place the binary `cargo-generate` in `~/.cargo/bin/`

## Usage

Standard usage is to pass a `--git` flag to `cargo generate` or short `cargo gen`. This will prompt you to enter the name of your project.

> NOTE: `cargo gen` requires an [cargo alias configuration](#cargo-gen---alias)

```sh
cargo generate --git https://github.com/githubusername/mytemplate.git
```

You can also pass the name of your project to the tool using the `--name` or `-n` flag:

```sh
cargo generate --git https://github.com/githubusername/mytemplate.git --name myproject
```

## git over ssh

New in version [0.7.0] is the support for both public and private and ssh git remote urls.
For example:

```sh
cargo generate --git git@github.com:rustwasm/wasm-pack-template.git --name mywasm
```

leads to the same result as:

```sh
cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name mywasm
```

as well as:

```sh
cargo generate --git rustwasm/wasm-pack-template --name mywasm
```

> NOTE: you can pass a custom ssh identity file with via `-i | --identity` like `-i ~/.ssh/id_rsa_other`

## http(s) proxy

New in version [0.7.0] is automatic proxy usage. So, if http(s)\_PROXY env variables are provided, they
will be used for cloning a http(s) template repository.

## Favorites

Favorite templates can be defined in a config file, that by default is placed at `$CARGO_HOME/cargo-generate`.
To specify an alternative configuration file, use the `--config <config-file>` option.

Each favorite template is specified in its own section, e.g.:

```toml
[favorites.demo]
description = "Demo template for cargo-generate"
git = "https://github.com/ashleygwilliams/wasm-pack-template"
branch = "master"
template_values = "<path to template-values file, relative to template, or absolute>"
```

Both `branch` and `description` are optional, and the branch may be overridden by specifying `--branch <branch>` on the command line.

When favorites are available, they can be generated simply by invoking:

```cli
cargo gen <favorite>
```

or slightly more involved:

```cli
cargo generate demo --branch master --name expanded_demo
```

> NOTE: when `<favorite>` is not defined in the config file, it is interpreted as a git repo like as if `--git <favorite>`

## Templates

Templates are git repositories whose files contain placeholders. The current
supported placeholders are:

-   `{{authors}}`

    this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration.

-   `{{project-name}}`

    this is supplied by either passing the `--name` flag to the command or working with the interactive CLI to supply a name.

-   `{{crate_name}}`

    the snake_case_version of `project-name`

-   `{{crate_type}}`

    this is supplied by either passing the `--bin` or `--lib` flag to the command line, contains either `bin` or `lib`, `--bin` is the default

-   `{{os-arch}}`

    contains the current operating system and architecture ex: `linux-x86_64`

Additionally, **all filters and tags** of the liquid template language are supported.
For more information, check out the [Liquid Documentation on `Tags` and `Filters`][liquid].

[liquid]: https://shopify.github.io/liquid

You can also add a `.genignore` file to your template. The files listed in the `.genignore` file
will be removed from the local machine when `cargo-generate` is run on the end user's machine.
The `.genignore` file is always ignored, so there is no need to list it in the `.genignore` file.

Here's a list of [currently available templates](TEMPLATES.md).
If you have a great template that you'd like to feature here, please [file an issue or a PR]!

[file an issue or a pr]: https://github.com/cargo-generate/cargo-generate/issues

### Example for `--bin` and `--lib`

A template could be prepared in a way to act as a binary or a library. For example the `Cargo.toml` might look like:

```toml
[package]
# the usual stuff

[dependencies]
{% if crate_type == "bin" %}
structopt = "0.3.21"
{% endif %}
# other general dependencies

{% if crate_type == "bin" %}
[[bin]]
path = "src/main.rs"
name = "{{crate_name}}-cli"
{% endif %}
```

Now a user of this template could decide weather they want the binary version by passing `--bin`
or use only the library version by passing `--lib` as a command line argument.

## Template defined placeholders

Sometimes templates need to make decisions. For example one might want to conditionally include some code or not.
Another use case might be that the user of a template should be able to choose out of provided options in an interactive way.
Also, it might be helpful to offer a reasonable default value that the user just simply can use.

Since version [0.6.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.6.0) it is possible to use placeholders in a `cargo-generate.toml` that is in the root folder of a template.
Here [an example](https://github.com/sassman/hermit-template-rs):

```toml
[placeholders.hypervisor]
type = "string"
prompt = "What hypervisor to use?"
choices = ["uhyve", "qemu"]
default = "qemu"

[placeholders.network_enabled]
type = "bool"
prompt = "Want to enable network?"
default = true
```

As you can see the `placeholders` configuration section accepts a table of keywords that will become the placeholder name.

In this example the placeholder `hypervisor` and `network_enabled` will become template variables and can be used like this:

```rs
{% if network_enabled %}
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    loop {
        let (conn, addr) = listener.accept().unwrap();
        println!("Incoming Connection from {}", addr);
        std::io::copy(&mut &conn, &mut &conn).unwrap();
    }
}
{% else %}
fn main() {
    println!("Hello Rusty Hermit 🦀");
}
{% endif %}
```

> Tip: similar to `dependencies` in the `Cargo.toml` file you can also list them as one liners:

```toml
[placeholders]
hypervisor = { type = "string", prompt = "What hypervisor to use?", choices = ["uhyve", "qemu"], default = "qemu" }
network_enabled = { type = "bool", prompt = "Want to enable network?", default = true }
```

### `prompt` property

The `prompt` will be used to display a question / message for this very placeholder on the interactive dialog when using the template.

```plain
🤷  What hypervisor to use? [uhyve, qemu] [default: qemu]:
```

### `type` property

A placeholder can be of type `string` or `bool`. Boolean types are usually helpful for conditionally behaviour in templates.

### `choices` property (optional)

A placeholder can come with a list of choices that the user can choose from.
It's further also validated at the time when a user generates a project from a template.

```toml
choices = ["uhyve", "qemu"]
```

### `default` property (optional)

A `default` property must mach the type (`string` | `bool`) and is optional. A default should be provided, to ease the interactive process.
As usual the user could press <enter> and the default value would simply be taken, it safes time and mental load.

```toml
default = 'qemu'
```

### `regex` property (optional)

A `regex` property is a string, that can be used to enforce a certain validation rule. The input dialog will keep repeating
until the user entered something that is allowed by this regex.

### Placeholder Examples

An example with a regex that allows only numbers

```toml
[placeholders]
phone_number = { type = "string", prompt = "What's your phone number?", regex = "[0-9]+" }
```

## Default values for placeholders from a file

For automation purposes the user of the template may provide provide a file containing the values for the keys in the template by using the `--template-values-file` flag.

The file should be a toml file containing the following (for the example template provided above):

```toml
[values]
hypervisor = "qemu"
network_enabled = true
```

## Include / Exclude

Templates support a `cargo-generate.toml`, with a "template" section that allows you to configure the files that will be processed by `cargo-generate`.
The behavior mirrors Cargo's Include / Exclude functionality, which is [documented here](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields-optional).
If you are using placeholders in a file name, and also wish to use placeholders in the contents of that file,
you should setup your globs to match on the pre-rename filename.

```toml
[template]
include = ["Cargo.toml"]
# include and exclude are exclusive, if both appear we will use include
exclude = ["*.c"]
```

## Cargo gen - alias

`cargo gen` requires an [cargo alias](https://doc.rust-lang.org/cargo/reference/config.html)
to be configured in your `$HOME/.cargo/config` like this:

```toml
[alias]
gen = "generate"
```

## License

Licensed under either of

-   Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
If you want to contribute to `cargo-generate`, please read our [CONTRIBUTING notes].

[contributing notes]: CONTRIBUTING.md
[0.7.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.7.0
