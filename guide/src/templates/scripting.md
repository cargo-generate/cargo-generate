# Init/Pre/Post Scripts

`cargo-generate` can run scripts in the [`Rhai`] language as part of the template expansion.

Doing so requires the template is configured to use hooks, which can be used at specific times
during template expansion.

To configure the use of hooks, write a `hooks` section in the `cargo-generate.toml` file.

```toml
[hooks]
#init = ["init-script.rhai"]
#pre = ["pre-script.rhai"]
#post = ["post-script.rhai"]
```

[`Rhai`]: https://rhai.rs/book/
