# Builtin placeholders

`cargo-generate` supports a number of builtin placeholders for use in templates.

These placeholders can be used directly in files using the [Liquid language][liquid], or from
[Rhai scripts][rhai] using the syntax:

```rhai
variable::get("placeholder name")
````

Builtin placeholders are:

* `authors`
    * this will be filled in by a function borrowed from Cargo's source code, that determines your information from
      Cargo's configuration. It will either be on the form `username <email>` or just plain `username`.
* `project-name`
    * this is supplied by either passing the `--name` flag to the command or working with the interactive CLI to supply
      a name. It can be provided in snake_case or dash-case, in all other cases it is converted to dash-case.
    * it can also be supplied via the environment variable `CARGO_GENERATE_VALUE_PROJECT_NAME` when running in `--silent` mode 
      > ⚠️ Note: the `--force` flag allows you to use the project name as it is given, without adjusting. Please use it carefully.
* `crate_name`
    * the snake_case_version of `project-name`
* `crate_type`
    * this is supplied by either passing the `--bin` or `--lib` flag to the command line, contains either `bin`
      or `lib`, `--bin` is the default
* `os-arch`
    * contains the current operating system and architecture ex: `linux-x86_64`
* `username`
    * this will be filled in by a function borrowed from Cargo's source code, that determines your information from
      Cargo's
      configuration.
* `within_cargo_project`
    * A boolean with the value `true` if the template is being expanded inside a `Cargo` project. It's
      a simple matter of whether `Cargo.toml` is present in any parent folder.
* `is_init`
    * A boolean that reflects the value of the `--init` parameter of `cargo-generate`.

## Overriding builtin placeholders

> Available since version [0.24.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.24.0)

Builtin placeholders can be overridden on the command line via `--define`, the
`--values-file`, or a `CARGO_GENERATE_VALUE_<NAME>` environment variable — the
same mechanisms used for template-defined placeholders. This is useful when the
auto-derived value (e.g. `authors` from local git config) should be pinned to a
stable value, for example when committing the expanded template to a
repository.

```sh
cargo generate --git … --define 'authors=The Project Authors'
```

An info message is logged whenever a builtin is overridden, so accidental
overrides remain visible.

## Usage example

```markdown
// README.md

This awesome crate `{{ crate_name }}` is brought to you by {{ authors }}.
```

[liquid]: https://shopify.github.io/liquid

[Rhai]: https://rhai.rs/book/
