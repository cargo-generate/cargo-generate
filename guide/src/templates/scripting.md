# Init/Pre/Post Scripts

`cargo-generate` can run scripts in the [`Rhai`] language as part of the template expansion.

Doing so requires the template is configured to use hooks, which can be used at specific times
during template expansion.

To configure the use of hooks, write a `hooks` section in the `cargo-generate.toml` file.

```toml
[hooks]
init = ["init-script.rhai"]
pre = ["pre-script.rhai"]
post = ["post-script.rhai"]
```

## Running system commands

Hooks can execute programs on the user's system. See [System commands] for
configuration, examples, permission controls, and the associated security
risks.

[`Rhai`]: https://rhai.rs/book/
[System commands]: scripting.system-commands.md
