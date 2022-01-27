# Conditional template settings

Using `cargo-generate.toml`, values and some [`Rhai`] syntax, the template author can make certain conditional decisions before expansion of the template.

`include`, `exclude`, `ignore` and `placeholders` can all be used in sections that are only used based upon the value of one or more values, possibly input by the user using the interactive prompt (if the values in question are defined as placeholders in the non-conditional section).

Using the following example, `cargo-generate` will ask for the `license`, and depending on the `--lib` | `--bin` flags it'll as for the `hypervisor` and `network_enabled` values. It will then continue to expand the template, ignoring the `src/main.rs` file (and thus excluding it from the output) in case `--lib` was specified.

The example is broken up in order to explain each section.

```toml
[template]
cargo_generate_version = ">=0.10.0"
# ignore = [ "..." ]
# include = [ "..." ]
# exclude = [ "..." ]
...
```

This first part declares that the template requires `cargo-generate` version 0.10 or higher. In this same section the template auther may also specify the following 3 lists:

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

The section should contain at least all variables used for any conditions (unless it's an automatic variable such as `crate_type`). All variables that are not specific to a condition are recommended to go here as well.

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

Here it has been choosen that the `src/main.rs` file must be ignored when the `crate_type` variable is equal to the string `"lib"`.

```toml
...
[conditional.'crate_type != "lib"'.placeholders]
hypervisor = { type = "string", prompt = "What hypervisor to use?", choices = ["uhyve", "qemu"], default = "qemu" }
network_enabled = { type = "bool", prompt = "Want to enable network?", default = true }
...
```

This block uses the same condition as the last, but it defines some extra placeholders - that is, is defines the variables `hypervisor` and `network_enabled`, so that `cargo-generate` may ask for their values.

> ⚠️ `cargo-generate` will ask for values using the placeholders defined in `[placeholders]` before evaluating the conditional sections.
>
> Placeholder values defined in conditional sections **cannot** be used to enable/disable further conditional sections, they can however still be used in the actual template!

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
