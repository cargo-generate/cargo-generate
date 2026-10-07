# Formatting

> Available since version [0.26.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.26.0)

`cargo-generate` formats the generated Rust files with `rustfmt`. Templates rarely produce
rustfmt-clean output, as conditionals and whitespace control are tedious to get right.

Formatting happens once the template has been expanded and moved to its final destination.

## What gets formatted

Only the files written by this run, and only those with a `.rs` extension.

This matters for `--init`, where a template is expanded into a directory that may already hold a
project. A template that adds a controller or a db module gets its own files formatted, while the
surrounding code is left as it was.

Each file is formatted against the edition of the cargo package it lands in, so a generated
workspace whose members use different editions is handled correctly.

## When nothing happens

A template that generates no `.rs` files has nothing to format. The same applies to Rust files
generated outside of a cargo project, as there is no manifest to read an edition from.

> ⚠️ NOTE: formatting never fails a generation. If the `rustfmt` component is missing, or `rustfmt`
> rejects a file it cannot parse, `cargo-generate` warns and leaves the generated files unformatted.

## Opting out as a template author

```toml
[template]
fmt = false
```

Use this when the template output is meant to be left exactly as written.

## Opting out as a user

```console
cargo generate --git https://github.com/example/template.git --no-fmt
```

`--no-fmt` wins over anything the template asks for.

## Opting back in explicitly

A template that has opted out may still request formatting by name:

```toml
[template]
fmt = false

[hooks]
post = ["fmt"]
```

`fmt` is not a script file, it is an identifier for the built-in step, and it takes precedence
over `fmt = false`. See [hook types](scripting.hook-types.md) for how it differs from a script hook.

The name `fmt` is reserved, so a hook script cannot use it. Any other name works, including
`fmt.rhai`. A template file named `fmt` is unaffected, and is templated and copied like any other
file.
