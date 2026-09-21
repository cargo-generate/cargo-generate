# Formatting

> Available since version [0.26.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.26.0)

`cargo-generate` formats the generated project with `rustfmt`. Templates rarely
produce rustfmt-clean output — conditionals and whitespace control make it hard
to get right — and formatting is the first thing most people do after
generating.

The step runs on the finished project in its final destination, after the
template has been expanded and moved there.

## What gets formatted

Only the files this generation wrote, and only the Rust ones. That matters for
`--init`, which expands a template into a directory that may already be
somebody's project: a template that adds a controller or a db module gets its
own files formatted, while the surrounding code is left exactly as it was.

Each file is formatted against the edition of the cargo package it lands in, so
a generated workspace whose members differ in edition is handled correctly.

## When it does nothing

A template that generates no Rust files has nothing to format, and the step is
a silent no-op. Templates that do not produce Rust code need no configuration.
The same applies to Rust files generated outside any cargo project — with no
manifest there is no edition to format them against.

Formatting never fails a generation. If the `rustfmt` component is not
installed, or rustfmt rejects a file it cannot parse, `cargo-generate` warns and
carries on with a generated project that is simply unformatted.

## Opting out as a template author

```toml
[template]
fmt = false
```

Use this when the template's output is meant to be left alone — for instance
when it ships deliberately-formatted fixtures.

## Opting out as a user

```console
cargo generate --git https://github.com/example/template.git --no-fmt
```

`--no-fmt` always wins, including over a template that asks for formatting.

## Opting back in explicitly

A template that has opted out can still request the built-in step by name:

```toml
[template]
fmt = false

[hooks]
post = ["cargo-fmt"]
```

`cargo-fmt` is not a script file. It is an identifier for the built-in step, and
it takes precedence over `fmt = false`. See
[Hook types](scripting.hook-types.md) for how it differs from a script hook.

Because the bare name is reserved, a hook script cannot be called exactly
`cargo-fmt`. Any other name works, `cargo-fmt.rhai` included. A template file
named `cargo-fmt` is unaffected and is templated and copied like any other.

This combination is uncommon — it exists so a template that opts out for one
reason can still be explicit about wanting formatting back.
