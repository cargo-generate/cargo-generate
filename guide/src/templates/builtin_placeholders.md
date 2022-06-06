# Builtin placeholders

`cargo-generate` supports a number of builtin placeholders for use in templates.

These placeholders can be used directly in files using the [Liquid language][liquid], or from
[Rhai scripts][rhai] using the syntax: `variable::get("placeholder name")`.

The current supported builtin placeholders are:

* `{{authors}}`

  this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration. It will either be on the form `username <email>` or just plain `username`.
* `{{project-name}}`

  this is supplied by either passing the `--name` flag to the command or working with the interactive CLI to supply a name.
* `{{crate_name}}`

  the snake_case_version of `project-name`
* `{{crate_type}}`

  this is supplied by either passing the `--bin` or `--lib` flag to the command line, contains either `bin` or `lib`, `--bin` is the default
* `{{os-arch}}`

  contains the current operating system and architecture ex: `linux-x86_64`
* `{{username}}`

  this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration.

* `{{cargo_embedded}}`

  A boolean with the value `true` if the template is being expanded inside a `Cargo` project. It's 
  a simple matter of whether `Cargo.toml` is present in any parent folder.

[liquid]: https://shopify.github.io/liquid
[Rhai]: https://rhai.rs/book/
