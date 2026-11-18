## Hook types

Rhai hook scripts run against a temporary copy of the template. Relative paths
in `file::` calls and the working directory for [system commands] refer to this
copy throughout generation.

### Init

- Init hooks run before project names are finalized and before template
  placeholders and conditional configuration are evaluated.
- The variables `crate_type`/`authors`/`username`/`os-arch` and `is_init` are available.
- The variable `project-name` *may* be available.

  And only if `cargo-generate` was called with the `--init` flag, in which case it is the raw user input.

- The variable `project-name` may be set - avoiding a user prompt!

  The variable will still be subject for case changes to fit with the rust/cargo expectations.

  The `--name` parameter still decides the final destination dir (together with the the `--init` flag),
  in order not to confuse the user.

### Pre

Pre hooks run after Cargo Generate resolves placeholders in `cargo-generate.toml`
and applies conditional configuration, and before Liquid expands template file
names and contents.

Use a pre hook to set variables for Liquid rendering or to rename or remove
files before expansion. Conditional configuration has already been applied at
this point. The [hooks example] uses a pre hook to replace `README.md` with
another file before expansion.

### Post

Post hooks run after Liquid expands file names and contents in the temporary
copy. Use a post hook to work with the rendered names or contents. Cargo Generate
copies the files to the destination after the post hooks succeed.

### Named hooks

> Available since version [0.26.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.26.0)

Most entries in `[hooks]` are paths to `.rhai` files. One is not: `fmt` names the built-in
[formatting](formatting.md) step rather than a script.

```toml
[hooks]
post = ["fmt"]
```

- The identifier only has meaning in `post`. In `init` or `pre` it is ignored with a warning.
- Unlike a script hook, it runs on the *final destination* rather than the template's working
  directory.
- Listing it re-enables formatting for a template that set `fmt = false`.
- The name is reserved, so a hook script cannot use it. Any other name, `fmt.rhai` included,
  behaves normally.

[hooks example]: https://github.com/cargo-generate/cargo-generate/tree/main/example-templates/hooks
[system commands]: scripting.system-commands.md
