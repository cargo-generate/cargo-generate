# Require `cargo-generate` version from template

> Available since version [0.9.0]

Using the supported `cargo-generate.toml` file, the template author may setup version requirements towards `cargo-generate`.

```toml
[template]
cargo_generate_version = ">=0.9.0"
```

The format for the version requirement is [documented here](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html).
