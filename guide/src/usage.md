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
cargo generate gl:username-on-gitlab/mytemplate # translates to https://gitlab.com/username-on-gitlab/mytemplate.git
# or for bitbucket.org
cargo generate bb:username-on-bitbucket/mytemplate # translates to https://bitbucket.org/username-on-bitbucket/mytemplate.git
# or for github.com
cargo generate gh:username-on-github/mytemplate # translates to https://github.com/username-on-github/mytemplate.git
# or for git.sr.ht
cargo generate sr:username-on-sourcehut/mytemplate # translates to https://git.sr.ht/~username-on-sourcehut/mytemplate (note the tilde)
```

Both will expand to the `https` urls of the repo with the suffix `.git` in the URL.

You can also pass the name of your project to the tool using the `--name` or `-n` flag:

```sh
cargo generate --git https://github.com/username-on-github/mytemplate.git --name myproject
```

## Templates in subfolders

If the repository or path specified for the template contains multiple templates (Any sub-folder that contains a `cargo-generate.toml` file), `cargo-generate` will ask for the specific folder to be used as the template.

Multiple *sub-templates* can also be configured in the `cargo-generate.toml` file like this:

```toml
[template]
sub_templates = ["folder1", "folder2"]
```

Doing so also sets the order when `cargo-generate` asks what to expand, while the first option will be the default.

The specific subfolder in the git repository may be specified on the command line like this:

```sh
cargo generate --git https://github.com/username-on-github/mytemplate.git <relative-template-path>
```

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

### git over ssh

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
cargo generate rustwasm/wasm-pack-template --name mywasm
```

### Authentication using `ssh-agent`

New in version [0.15.1] is the `ssh-agent` usage for password protected keys. It's also the default mechanism on Windows.

#### On Windows

`ssh-agent` is the default and also **the only** possibility to get git+ssh working.
Please follow [this guide](https://github.com/cargo-generate/cargo-generate/discussions/653) to get `ssh-agent` configured on windows.

#### On *Nix and macOS

If you omit the identity file (read the next paragraph) **OR** a provided identity file does not exist, then the default is to use `ssh-agent`.

### Ssh identity file (private key)

However, if you use a different file, you can pass a custom ssh identity with via `-i | --identity` like `-i ~/.ssh/id_rsa_other` as argument.

> ⚠️ NOTE: password protected private keys are **NOT** supported, you have to use `ssh-agent` and omit the `-i` argument (see above).

If you permanently want to use a custom identity file, you can configure it in the cargo-generate config file like this:

```toml
# an extract of ~/.cargo/cargo-generate.toml
[defaults]
# note that `~/` and `$HOME/` are going to be expanded to the full path seamlessly
ssh_identity = "~/.ssh/id_rsa_other"
# that is equivalent to
ssh_identity = "$HOME/.ssh/id_rsa_other"
# that is equivalent to
ssh_identity = "/home/john/.ssh/id_rsa_other"
```

> ⚠️ NOTE: that the cli argument `-i` always overrules the `ssh_identity` from the config file.

## Http(s) proxy

New in version [0.7.0] is automatic proxy usage. So, if http(s)\_PROXY env variables are provided, they
will be used for cloning a http(s) template repository.

[0.7.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.7.0
[0.9.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.9.0
[0.15.1]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.15.1
[VSCode]: https://code.visualstudio.com
[`Rhai`]: https://rhai.rs/book/
[Rhai language extension]: https://marketplace.visualstudio.com/items?itemName=rhaiscript.vscode-rhai

## `.gitconfig` and `insteadOf` configuration

⚠️ New in version [0.22.0]

git supports a magic trick to rewrite urls on the fly. This is done by adding a `url.<base>.insteadOf` configuration to your `.gitconfig` file.

In cargo-generate this is supported as well.

For example, if you prefer the ssh over the https urls and you want to use `cargo-generate` with it, you can add the following to your `.gitconfig`:

```.gitconfig
# ~/.gitconfig

[url "git@github.com:"]
insteadOf = https://github.com/
```

and then you can use `cargo-generate` with the `https` url:

```sh
RUST_LOG=debug cargo generate https://github.com/Rahix/avr-hal-template.git

🔧   gitconfig 'insteadOf' lead to this url: git@github.com:Rahix/avr-hal-template.git

...
```

In this case please notice the ssh url is `git@github.com:` if you prefer the more explicit notation you can also write it like this:

```.gitconfig
# ~/.gitconfig

[url "ssh://git@github.com/"]
insteadOf = https://github.com/
```

that would lead to the same result, with slightly different url:

```sh
RUST_LOG=debug cargo generate https://github.com/Rahix/avr-hal-template.git

🔧   gitconfig 'insteadOf' lead to this url: ssh://git@github.com/Rahix/avr-hal-template.git

...
```

> ⚠️ NOTE: `RUST_LOG=debug` would allow you to see the rewritten url in the output.

In cases where you have a different `.gitconfig` location, you can use the `--gitconfig` argument to specify the path to the `.gitconfig` file, like this:

```sh
$ cd /path/to/my/workspace
$ cat .gitconfig
[url "git@github.com:"]
insteadOf = https://github.com/

$ cargo generate --gitconfig ./.gitconfig https://github.com/Rahix/avr-hal-template.git
```
