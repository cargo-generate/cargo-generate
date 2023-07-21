# Include / Exclude

Templates support a `cargo-generate.toml`, with a "template" section that allows you to configure the files that will be processed by `cargo-generate`.
The behavior mirrors Cargo's Include / Exclude functionality, which is [documented here](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields-optional).
If you are using placeholders in a file name, and also wish to use placeholders in the contents of that file,
you should setup your globs to match on the pre-rename filename.

```toml
[template]
include = ["Cargo.toml"]
# include and exclude are exclusive, if both appear we will use include
exclude = ["*.c"]
```

> ⚠️ NOTE: `exclude` only makes `cargo-generate` ignore any `liquid` tags in the file. In order to exclude a file from being copied to the final dir, see [ignoring files](ignoring.md).

The `cargo-generate.toml` file should be placed in the root of the template. If using the `subfolder` feature, the root is the `subfolder` inside the repository, though `cargo-generate` will look for the file in all parent folders until it reaches the repository root.
