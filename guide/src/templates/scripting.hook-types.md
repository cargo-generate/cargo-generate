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


### Final

> Available since version 0.27.0

Final hooks run in the generated project's destination after files have been copied,
workspace membership and formatting have been handled, and Git has been initialized
when requested. Use them for tasks that depend on the destination, such as
`direnv allow` or an initial Git commit.

```toml
[hooks]
final = ["final-script.rhai"]
```

Scripts and their imports are read from the temporary template, and configured hook
scripts are excluded from the generated output. The `file` functions and system
commands operate in the destination directory. Template variables remain available.
[System commands](scripting.system-commands.md) retain their usual confirmation and
`--allow-commands` behavior.

Unlike pre and post hooks, a failing final hook leaves the generated project in place.
With `--init`, final hooks can modify existing destination files; those changes are not
rolled back if a later hook fails. With `--test`, final hooks run in the temporary
expanded project before the test command, rather than in the template author's directory.

### Named hooks

> Available since version [0.26.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.26.0)

Most entries in `[hooks]` are paths to `.rhai` files. One is not: `fmt` names the built-in
[formatting](formatting.md) step rather than a script.

```toml
[hooks]
post = ["fmt"]
```

- The identifier only has meaning in `post`. In `init`, `pre` or `final` it is ignored with a warning.
- The formatting step runs on the *final destination* rather than the temporary template,
  before any final script hooks.
- Listing it re-enables formatting for a template that set `fmt = false`.
- The name is reserved, so a hook script cannot use it. Any other name, `fmt.rhai` included,
  behaves normally.

[`Rhai`]: https://rhai.rs/book/
