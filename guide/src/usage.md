# Usage
Standard usage is to pass a `--git` flag to `cargo generate` or short `cargo gen`. This will prompt you to enter the name of your project.

> ⚠️ NOTE: `cargo gen` requires a [cargo alias configuration](#cargo-gen---alias)

```sh
cargo generate username-on-github/mytemplate
# is the same as 
cargo generate gh:username-on-github/mytemplate
# is the same as 
cargo generate --git https://github.com/username-on-github/mytemplate.git
```

If you have your templates not GitHub then you can leverage the lazy abbreviation prefixes:
```sh
# for gitlab.com
cargo generate gl:username-on-gitlab/mytemplate
# or for bitbucket.org
cargo generate bb:username-on-bitbucket/mytemplate
# or for github.com 
cargo generate gh:username-on-github/mytemplate
```

Both will expand to the `https` urls of the repo with the suffix `.git` in the URL.

You can also pass the name of your project to the tool using the `--name` or `-n` flag:

```sh
cargo generate --git https://github.com/username-on-github/mytemplate.git --name myproject
```

## Templates in subfolders

If the git repository contains multiple templates, the specific subfolder in the git repository may be specified like this:

```sh
cargo generate --git https://github.com/username-on-github/mytemplate.git <relative-template-path>
```

> ⚠️ NOTE: The specified `relative-template-path` will be used as the actual template root, whether or not this is actually true!

> ⚠️ NOTE: When using the `subfolder` feature, `cargo-generate` will search for the `cargo-generate.toml` file in the subfolder first, traversing back towards the template root in case it is not found.

## Generating into current dir

If the user wants to generate a template straight into the current folder, without creating a subfolder for the contents and without attempting to initialize a `.git` repo or similar, the `--init` flag can be used.

```sh
cargo generate --init --git https://github.com/username-on-github/mytemplate.git
```

> ⚠️ NOTE: `cargo-generate` will not allow any existing files to be overwritten and will fail to generate any files should there be any conflicts.

## Generating using a local template

You can generate a project using a local template via the `--path` flag:

```sh
git clone https://github.com/username-on-github/mytemplate.git $HOME/mytemplate # Clone any template
cargo generate --path $HOME/mytemplate # Use it locally
```

> ⚠️ NOTE: `cargo-generate` will not allow to use the association `--path` and `--git` flags.

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

> ⚠️ NOTE: you can pass a custom ssh identity file with via `-i | --identity` like `-i ~/.ssh/id_rsa_other`

## http(s) proxy

New in version [0.7.0] is automatic proxy usage. So, if http(s)\_PROXY env variables are provided, they
will be used for cloning a http(s) template repository.


[0.7.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.7.0
[0.9.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.9.0
[VSCode]: https://code.visualstudio.com
[`Rhai`]: https://rhai.rs/book/
[Rhai language extension]: https://marketplace.visualstudio.com/items?itemName=rhaiscript.vscode-rhai
