# Git over SSH

Both SSH URL forms are supported. Note that the `ssh://` form uses a path
separator, while the `git@` shorthand uses a colon between host and org:

```raw
git@github.com:rustwasm/wasm-pack-template.git

# vs

ssh://git@github.com/rustwasm/wasm-pack-template.git
```

Either one can also be used as the right-hand side of `.gitconfig`
`insteadOf` — see the next chapter.

```sh
cargo generate --git git@github.com:rustwasm/wasm-pack-template.git --name mywasm
```

## How authentication works

Since the migration to [`gix`], cargo-generate delegates all SSH concerns to
the system `ssh` binary. In practice that means:

* `ssh-agent` is picked up automatically wherever the OS exposes it.
* `~/.ssh/config` is honored, including per-host `IdentityFile`,
  `IdentitiesOnly`, `ProxyJump`, and friends.
* Default identity discovery is whatever your `ssh` uses — typically
  `~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, etc.
* Passphrase prompts come from `ssh` (or the agent) directly, so they look
  and behave exactly like a plain `git clone`.

No cargo-generate–specific configuration is needed for the common case: if
`git clone <ssh-url>` works in your shell, so does
`cargo generate --git <ssh-url>`.

### On Windows

`ssh-agent` ships as an optional service with modern Windows. Follow
[this guide](https://github.com/cargo-generate/cargo-generate/discussions/653)
for one-time setup; once it's running, cargo-generate uses it transparently.

## Custom SSH identity file (private key)

If you need a specific key for a single invocation, pass it with
`-i` / `--identity`:

```sh
cargo generate -i ~/.ssh/id_rsa_other --git git@github.com:org/template.git
```

Under the hood this becomes an in-memory `core.sshCommand = ssh -i <path>`
override — equivalent to running git with `GIT_SSH_COMMAND=ssh -i <path>`.
Passphrase prompts (if any) come from `ssh` directly.

For a persistent choice, `~/.ssh/config` is usually the cleanest option:

```
Host github.com
    IdentityFile ~/.ssh/id_rsa_other
    IdentitiesOnly yes
```

Alternatively, configure it in the cargo-generate config file:

```toml
# an extract of ~/.cargo/cargo-generate.toml
[defaults]
# note that `~/` and `$HOME/` are expanded to the full path seamlessly
ssh_identity = "~/.ssh/id_rsa_other"
# equivalent to
ssh_identity = "$HOME/.ssh/id_rsa_other"
# equivalent to
ssh_identity = "/home/john/.ssh/id_rsa_other"
```

> ⚠️ NOTE: the CLI argument `-i` always overrules `ssh_identity` from the
> config file.

[`gix`]: https://github.com/GitoxideLabs/gitoxide
