# Conditional template settings

Using `cargo-generate.toml`, values and some [`Rhai`] syntax, the template author can make certain conditional decisions before expansion of the template.

`include`, `exclude`, `ignore` and `placeholders` can all be used in sections that are only used based upon the value of one or more values, possibly input by the user using the interactive prompt (if the values in question are defined as placeholders in the non-conditional section).

The key inside `conditional.'...'` is a TOML literal string containing a single
[`Rhai`] expression. Use single quotes for the TOML key so double quotes in the
expression, such as `platform == "web"`, need no escaping. The expression must
evaluate to a boolean. The block applies when the result is `true`.

Use Rhai operators such as `==`, `&&`, and `||`. A condition can also call a
Rhai function or method, such as `.contains()` on an array literal or array
placeholder. Conditions use Rhai's [expression evaluator]. Use a hook script
for statements such as `let` or for cargo-generate's
[hook extensions](scripting.rhai-extensions.md).
Evaluation errors, including an undefined variable or a non-boolean result,
skip the block for that pass.

## Share a placeholder across multiple choices

In this `cargo-generate.toml`, one condition introduces the same `backend`
placeholder for both `fullstack` and `liveview`:

```toml
[placeholders.platform]
type = "string"
prompt = "What platform are you targeting?"
choices = ["web", "fullstack", "liveview"]
default = "web"

[conditional.'["fullstack", "liveview"].contains(platform)'.placeholders.backend]
type = "string"
prompt = "What backend framework are you using?"
choices = ["Axum", "Salvo"]
default = "Axum"

[conditional.'backend == "Axum"']
ignore = ["salvo-notes.txt"]
```

With `platform = "web"`, there is no backend prompt and the template keeps
`salvo-notes.txt`. With either backend-capable platform, cargo-generate first
resolves `backend`, then evaluates the remaining conditions again. Choosing
`Axum` skips `salvo-notes.txt`, and choosing `Salvo` keeps it.

## Combine conditional settings

In this example, `cargo-generate` asks for the `license` and, depending on the
`--lib` or `--bin` flag, asks for the `hypervisor` and `network_enabled` values.
When `--lib` is specified, it omits `src/main.rs` from the generated project.

The example is broken up in order to explain each section.

```toml
[template]
cargo_generate_version = ">=0.10.0"
# ignore = [ "..." ]
# include = [ "..." ]
# exclude = [ "..." ]
...
```

This first part declares that the template requires `cargo-generate` version 0.10 or higher. In this same section the template author may also specify the following 3 lists:

* `ignore` Files/folders on this list will be ignored entirely and are not included in the final output.
* `include` These files will be processed for `Liquid` syntax by the template engine.
* `exclude` These files will *not* be processed for any `liquid` syntax. The files will be in the final output.

```toml
...
[placeholders]
license = { type = "string", prompt = "What license to use?", choices = ["MIT", "Unrestricted"], default = "MIT" }
...
```

This is the section for the *default* placeholders. These are variable definitions that `cargo-generate` knows about and will query for if they are not provided e.g. on the commandline (see [Default-values-for-placeholders]).

Define variables used by the first round of conditions here, unless they are
automatic variables such as `crate_type`. This is also where to put
placeholders that should be available regardless of the selected conditions.

Here we simply define a variable `license` for selecting the desired license type.

```toml
...
[conditional.'crate_type == "lib"']
ignore = [ "src/main.rs" ]
# include = [ "..." ]
# exclude = [ "..." ]
...
```

This is a conditional block.

Here it has been chosen that the `src/main.rs` file must be ignored when the `crate_type` variable is equal to the string `"lib"`.

```toml
...
[conditional.'crate_type != "lib"'.placeholders]
hypervisor = { type = "string", prompt = "What hypervisor to use?", choices = ["uhyve", "qemu"], default = "qemu" }
network_enabled = { type = "bool", prompt = "Want to enable network?", default = true }
...
```

When `crate_type` is anything other than `"lib"`, this block adds
`hypervisor` and `network_enabled` as placeholders.

> `cargo-generate` resolves the top-level `[placeholders]` before evaluating
> conditional sections. When a matching section introduces more placeholders,
> it resolves those values and evaluates the remaining sections again.

```toml
...
[conditional.'license == "MIT"']
ignore = [ "LICENSE-UNRESTRICTED.txt" ]
# include = [ "..." ]
# exclude = [ "..." ]

[conditional.'license == "Unrestricted"']
ignore = [ "LICENSE-MIT.txt" ]
# include = [ "..." ]
# exclude = [ "..." ]
```

This last conditional block is simply to ignore the unneeded license files, based upon the users choice for the `license` variable.

> ⚠️ Note that `include` and `exclude` are still mutually exclusive even if they are in different, but included, conditional sections.

[`Rhai`]: https://rhai.rs/book/
[expression evaluator]: https://docs.rs/rhai/latest/rhai/struct.Engine.html#method.eval_expression
