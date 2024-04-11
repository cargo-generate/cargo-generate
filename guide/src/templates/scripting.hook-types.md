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


[`Rhai`]: https://rhai.rs/book/
