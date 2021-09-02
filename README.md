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

> :warning: NOTE: `vendored-openssl` requires the following packages to be installed:
>  - libssl-dev
>  - gcc
>  - m4
>  - ca-certificates
>  - make
>  - perl

```sh
cargo install cargo-generate --features vendored-openssl
```

### Using `pacman` (Arch Linux)

`cargo-generate` can be installed from the [community repository](https://archlinux.org/packages/community/x86_64/cargo-generate/) for Arch Linux:

```sh
pacman -S cargo-generate
```

### Manual Install:

1. Download the binary tarball for your platform from our [releases page](https://github.com/cargo-generate/cargo-generate/releases).

2. Unpack the tarball and place the binary `cargo-generate` in `~/.cargo/bin/`

## Usage

Standard usage is to pass a `--git` flag to `cargo generate` or short `cargo gen`. This will prompt you to enter the name of your project.

> :warning: NOTE: `cargo gen` requires a [cargo alias configuration](#cargo-gen---alias)

```sh
cargo generate --git https://github.com/githubusername/mytemplate.git
```

You can also pass the name of your project to the tool using the `--name` or `-n` flag:

```sh
cargo generate --git https://github.com/githubusername/mytemplate.git --name myproject
```

#### Templates in subfolders

If the git repository contains multiple templates, the specific subfolder in the git repository may be specified like this:

```sh
cargo generate --git https://github.com/githubusername/mytemplate.git <relative-template-path>
```

> :warning: NOTE: The specified `relative-template-path` will be used as the actual template root, whether or not this is actually true!

> :warning: NOTE: When using the `subfolder` feature, `cargo-generate` will search for the `cargo-generate.toml` file in the subfolder first, traversing back towards the template root in case it is not found.

#### Generating into current dir

If the user wants to generate a template straight into the current folder, without creating a subfolder for the contents and without attempting to initialize a `.git` repo or similar, the `--init` flag can be used.

```sh
cargo generate --init --git https://github.com/githubusername/mytemplate.git
```

> :warning: NOTE: `cargo-generate` will not allow any existing files to be overwritten and will fail to generate any files should there be any conflicts.

#### Generating using a local template

You can generate a project using a local template via the `--path` flag:

```sh
git clone https://github.com/githubusername/mytemplate.git $HOME/mytemplate # Clone any template
cargo generate --path $HOME/mytemplate # Use it locally
```

> :warning: NOTE: `cargo-generate` will not allow to use the association `--path` and `--git` flags.

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

> :warning: NOTE: you can pass a custom ssh identity file with via `-i | --identity` like `-i ~/.ssh/id_rsa_other`

## http(s) proxy

New in version [0.7.0] is automatic proxy usage. So, if http(s)\_PROXY env variables are provided, they
will be used for cloning a http(s) template repository.

## Favorites

Favorite templates can be defined in a config file, that by default is placed at `$CARGO_HOME/cargo-generate.toml` or `$CARGO_HOME/cargo-generate`.
To specify an alternative configuration file, use the `--config <config-file>` option.

> :warning: NOTE: A relative `--config` option, will be relative to the template root during expansion.

Each favorite template is specified in its own section, e.g.:

```toml
[favorites.demo]
description = "<optional description, visible with --list-favorites>"
git = "https://github.com/ashleygwilliams/wasm-pack-template"
branch = "<optional-branch>"
subfolder = "<optional-subfolder>"
```

Values may be overridden using the CLI arguments of the same names (e.g. `--subfolder` for the `subfolder` value).

When favorites are available, they can be generated simply by invoking:

```cli
cargo gen <favorite>
```

or slightly more involved:

```cli
cargo generate demo --branch mybranch --name expanded_demo --subfolder myfolder
```

> :warning: NOTE: when `<favorite>` is not defined in the config file, it is interpreted as a git repo like as if `--git <favorite>`

## Templates

Templates are git repositories whose files contain placeholders. The current
supported placeholders are:

- `{{authors}}`

  this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration.
- `{{project-name}}`

  this is supplied by either passing the `--name` flag to the command or working with the interactive CLI to supply a name.
- `{{crate_name}}`

  the snake_case_version of `project-name`
- `{{crate_type}}`

  this is supplied by either passing the `--bin` or `--lib` flag to the command line, contains either `bin` or `lib`, `--bin` is the default
- `{{os-arch}}`

  contains the current operating system and architecture ex: `linux-x86_64`

Additionally, **all filters and tags** of the liquid template language are supported.
For more information, check out the [Liquid Documentation on `Tags` and `Filters`][liquid].

[liquid]: https://shopify.github.io/liquid

You can use those placeholders in the file and directory names of the generated project.
For example, for a project named `awesome`, the filename `{{project_name}}/{{project_name}}.rs` will be transformed to `awesome/awesome.rs` during generation.
Only files that are **not** listed in the exclude settings will be templated.

> :warning: NOTE: invalid characters for a filename or directory name will be sanitized after template substitution. Invalid is e.g. `/` or `\`.

> :warning: **Deprecated** in favor of using [ignore in `cargo-generate.toml`](#Ignoring-files)
>
> You can also add a `.genignore` file to your template. The files listed in the `.genignore` file
> will be removed from the local machine when `cargo-generate` is run on the end user's machine.
> The `.genignore` file is always ignored, so there is no need to list it in the `.genignore` file.

### Templates by the community

It's encouraged to classify your template repository [with a GitHub topic](https://docs.github.com/en/github/administering-a-repository/managing-repository-settings/classifying-your-repository-with-topics) labeled `cargo-generate`.

So that every developer can find the template via [cargo-generate topic on GitHub](https://github.com/topics/cargo-generate).

If you have a great template, please tag your repository with the topic [and tweet about it](https://twitter.com/intent/tweet?text=See%20my%20new%20%23cargogenerate%20%23template%20%0A%0A%3E%20your%20link%20goes%20here) by including the hashtag [`#cargogenerate`](https://twitter.com/search?q=%23cargogenerate&src=typed_query) (since twitter does not support hashtags with `-`).

> :warning: Note: the list of [currently available templates](TEMPLATES.md) is still available, but is now deprecated.

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

> :bulb: Tip: similar to `dependencies` in the `Cargo.toml` file you can also list them as one liners:
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

For automation purposes the user of the template may provide the values for the keys in the template using one or more of the following methods.

The methods are listed by falling priority.

#### `--define` or `-d` flag

The user may specify variables individually using the `--define` flag.

```sh
cargo generate template-above -n project-name -d hypervisor=qemu -d network_enabled=true
```

#### <a name="valuesfile"></a> `--template_values_file` flag

The user of the template may provide a file containing the values for the keys in the template by using the `--template-values-file` flag.

> :warning: NOTE: A relative path will be relative to current working dir, which is *not* inside the expanding template!

The file should be a toml file containing the following (for the example template provided above):

```toml
[values]
hypervisor = "qemu"
network_enabled = true
```

#### Individual values via environment variables

Variables may be specified using environment variables. To do so, set the env var `CARGO_GENERATE_VALUE_<variable key>` to the desired value.

```sh
set CARGO_GENERATE_VALUE_HYPERVISOR=qemu
set CARGO_GENERATE_VALUE_NETWORK_ENABLED=true
cargo generate template-above
```

> :warning: Windows does not support mixed case environment variables. Internally, `cargo-generate` will ensure the variable name is all lowercase. For that reason, it is strongly recommended that template authors only use lowercase variable/placeholder names.

#### Template values file via environment variable

The user may use the environment variable `CARGO_GENERATE_TEMPLATE_VALUES` to specify a file with default values.

For the file format, see [above](#valuesfile)

#### Default values

Default values may be specified in the config file (specified with the `--config` flag, or in the default config file `$CARGO_HOME/cargo-generate`)

**Example config file:**

```toml
[values]
placeholder1 = "default value"

[favorites.my_favorite]
git = "https://github.com/githubusername/mytemplate.git"

[favorites.my_favorite.values]
placeholder1 = "default value overriding the default"
placeholder2 = "default value for favorite"
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

> :warning: NOTE: `exclude` only makes `cargo-generate` ignore any `liquid` tags in the file. In order to exclude a file from being copied to the final dir, see [ignoring files](#Ignoring-files).

The `cargo-generate.toml` file should be placed in the root of the template. If using the `subfolder` feature, the root is the `subfolder` inside the repository, though `cargo-generate` will look for the file in all parent folders until it reaches the repository root.

## Ignoring files

The template author may choose to ignore files completely, by including an `ignore` list in the `cargo-generate.toml` file.

Example:

```toml
[template]
ignore = [ 
  "file",
  "or folder",
  "to be ignored" 
]
```

Both files and folders may be ignored using this method, but currently wildcards are **not supported**. 

## Require `cargo-generate` version from template

> Available since version [0.9.0]

Using the supported `cargo-generate.toml` file, the template author may setup version requirements towards `cargo-generate`.

```toml
[template]
cargo_generate_version = ">=0.9.0"
```

The format for the version requirement is [documented here](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html).

## Conditional template settings.

Using `cargo-generate.toml`, values and some [`Rhai`](https://rhai.rs/book/) syntax, the template author can make certain conditional decisions before expansion of the template.

`include`, `exclude`, `ignore` and `placeholders` can all be used in sections that are only used based upon the value of one or more values, possibly input by the user using the interactive prompt (if the values in question are defined as placeholders in the non-conditional section).

Example:

```toml
[template]
cargo_generate_version = ">=0.10.0"
# ignore = [ "..." ]
# include = [ "..." ]
# exclude = [ "..." ]

[placeholders]
license = { type = "string", prompt = "What license to use?", choices = ["MIT", "Unrestricted"], default = "MIT" }

[conditional.'crate_type == "lib"']
ignore = [ "src/main.rs" ]
# include = [ "..." ]
# exclude = [ "..." ]

[conditional.'crate_type != "lib"'.placeholders]
hypervisor = { type = "string", prompt = "What hypervisor to use?", choices = ["uhyve", "qemu"], default = "qemu" }
network_enabled = { type = "bool", prompt = "Want to enable network?", default = true }

[conditional.'license == "MIT"']
ignore = [ "LICENSE-UNRESTRICTED.txt" ]
# include = [ "..." ]
# exclude = [ "..." ]

[conditional.'license == "Unrestricted"']
ignore = [ "LICENSE-MIT.txt" ]
# include = [ "..." ]
# exclude = [ "..." ]
```

Using the above example `cargo-generate` will ask for the `license`, and depending on the `--lib` | `--bin` flags it'll as for the `hypervisor` and `network_enabled` values. It will then continue to expand the template, ignoring the `src/main.rs` file (and thus excluding it from the output) in case `--lib` was specified.

> :warning: Note that `include` and `exclude` are still mutually exclusive even if they are in different, but included, conditional sections.

> :warning: `cargo-generate` will ask for values using the placeholders defined in `[placeholders]` before evaluating the conditional sections. Placeholder values defined in conditional sections **cannot** be used to enable/disable further conditional sections, they can however still be used in the actual template!

## Cargo gen - alias

`cargo gen` requires a [cargo alias](https://doc.rust-lang.org/cargo/reference/config.html)
to be configured in your `$HOME/.cargo/config` like this:

```toml
[alias]
gen = "generate"
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
[0.7.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.7.0
[0.9.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.9.0
