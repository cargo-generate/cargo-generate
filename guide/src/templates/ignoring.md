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
