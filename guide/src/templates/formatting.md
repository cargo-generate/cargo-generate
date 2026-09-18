# Formatting

> Available since version [0.26.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.26.0)

`cargo-generate` runs `cargo fmt` on the generated project. Templates rarely
produce rustfmt-clean output — conditionals and whitespace control make it hard
to get right — and formatting is the first thing most people do after
generating.

The step runs on the finished project in its final destination, after the
template has been expanded and moved there.

## When it does nothing

A generated project without a `Cargo.toml` has nothing to format, and the step
is a silent no-op. Templates that do not produce Rust crates need no
configuration.

Formatting never fails a generation. If `cargo` is not on `PATH`, the `rustfmt`
component is not installed, or rustfmt rejects a file, `cargo-generate` warns
and carries on with a generated project that is simply unformatted.

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

This combination is uncommon — it exists so a template that opts out for one
reason can still be explicit about wanting formatting back.
