# Ignoring files

The template author may choose to ignore files completely, by including an `ignore` list in the `cargo-generate.toml` file.

Example:

```toml
[template]
ignore = [ 
  "file",
  "or folder",
  "to be ignored" 
]
```

Both files and folders may be ignored using this method, but currently wildcards are **not supported**.

Note that `cargo-generate` checks for which files to ignore after the removal of any `.liquid` file extensions.
Meaning; Setting `ignore` to `["file.txt"]` will result in the ignoring of a file named `file.txt.liquid`.
