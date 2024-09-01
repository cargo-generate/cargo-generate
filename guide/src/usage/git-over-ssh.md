# git over ssh

New in version [0.7.0] is the support for both public and private and ssh git remote urls.
New in version [0.22.0] is the support for `ssh-agent` on Windows and interactively asking for passphrase for password protected keys on *Nix and macOS.

There are 2 different git over ssh urls, one with the `git@` prefix and one with the `ssh://` prefix. Both are supported. Please note that the `ssh://` prefix url uses a path where the `git@` prefix uses a colon at the user/github-org level. For example:

```raw
git@github.com:rustwasm/wasm-pack-template.git

# vs

ssh://git@github.com/rustwasm/wasm-pack-template.git
```

Both those urls can also be used in the `.gitconfig` insteadOf configuration, see more in the section below.

```sh
cargo generate --git git@github.com:rustwasm/wasm-pack-template.git --name mywasm
```

## Authentication using `ssh-agent`

New in version [0.15.1] is the `ssh-agent` usage for password protected keys. It's also the default mechanism on Windows.

### On Windows

`ssh-agent` is the default and also **the only** possibility to get git+ssh working.
Please follow [this guide](https://github.com/cargo-generate/cargo-generate/discussions/653) to get `ssh-agent` configured on windows.

### On *Nix and macOS

If you omit the identity file (read the next paragraph) **OR** a provided identity file does not exist, then the default is to use `ssh-agent`.

## Ssh identity file defaults

Since version [0.22.0] the default mechanism on unix/macOS is to use the function [`add_default_ssh_keys`](https://github.com/de-vri-es/auth-git2-rs/blob/fd6502e20b9e82063b950ac9e32f6454341e71f2/src/lib.rs#L344) that adds a couple of default keys to look up first.

```
// the list used..
let candidates = [
			"id_rsa",
			"id_ecdsa,",
			"id_ecdsa_sk",
			"id_ed25519",
			"id_ed25519_sk",
			"id_dsa",
		];
```

## Custom ssh identity file (private key)

However, if you use a different file, you can pass a custom ssh identity with via `-i | --identity` like `-i ~/.ssh/id_rsa_other` as argument.

If the file is passphrase protected cargo-generate will ask for the passphrase interactively.

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
