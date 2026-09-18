## Hook types

### Init

- Init hooks are executed before anything else.
- The variables `crate_type`/`authors`/`username`/`os-arch` and `is_init` are available.
- The variable `project-name` *may* be available.

  And only if `cargo-generate` was called with the `--init` flag, in which case it is the raw user input.

- The variable `project-name` may be set - avoiding a user prompt!

  The variable will still be subject for case changes to fit with the rust/cargo expectations.

  The `--name` parameter still decides the final destination dir (together with the the `--init` flag),
  in order not to confuse the user.

### Pre

- Pre hooks are run *after all placeholders mentioned in cargo-generate.toml has been resolved*.
- The hooks are free to add additional variables, but its too late to influence the conditional system.

  This is a side effect of conditionals influencing the hooks - so placeholders need to be evaluated before the hooks are known.

### Post

- Post hooks are run after template expansion, but *before final output is moved to the final destination*.

Why not later? Security, and the fact that a failing script still causes no errors in the users destination.


### Named hooks

> Available since version [0.26.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.26.0)

Most entries in `[hooks]` are paths to `.rhai` files. One is not: `cargo-fmt`
names the built-in formatting step rather than a script.

```toml
[hooks]
post = ["cargo-fmt"]
```

It only has meaning in `post`; in `init` or `pre` it is ignored with a warning.
Unlike a script hook it runs on the *final destination* rather than the
template's working directory, so it is unaffected by the temp directory that
script hooks see.

Listing it re-enables formatting for a template that set `fmt = false`. See
[Formatting](formatting.md).

[`Rhai`]: https://rhai.rs/book/
