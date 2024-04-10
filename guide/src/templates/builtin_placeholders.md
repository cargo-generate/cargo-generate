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
      a name. It can be in snake_case or dash-case. In other cases it is
      converted to dash-case.
      Note: the `--force` flag allows to use the project name as it is given, without adjusting. Pls use it with care.
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

## Usage example

```markdown
// README.md

This awesome crate `{{ crate_name }}` is brought to you by {{ authors }}.
```

[liquid]: https://shopify.github.io/liquid

[Rhai]: https://rhai.rs/book/
