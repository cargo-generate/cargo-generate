# Favorites

Favorite templates can be defined in a config file, that by default is placed at `$CARGO_HOME/cargo-generate.toml` or `$CARGO_HOME/cargo-generate`.
To specify an alternate configuration file, use the `--config <config-file>` option.

> ⚠️ NOTE: A relative `--config` option, will be relative to the template root during expansion.

Each favorite template is specified in its own section, e.g.:

```toml
[favorites.demo]
description = "<optional description, visible with --list-favorites>"
git = "https://github.com/ashleygwilliams/wasm-pack-template"
branch = "<optional-branch>"
subfolder = "<optional-subfolder>"
vcs = "<optional: None|Git>"
init = optional true|false
overwrite = optional true|false
```

Values may be overridden using the CLI arguments of the same names (e.g. `--subfolder` for the `subfolder` value).

**Note:** Specifying `init = true` has the effect of forcing the template to exhibit behaviour as if `--init` is specified on the
commandline, as there is no counter-option!

**Note:** Specifying `overwrite = true` has the effect of allowing the template to always overwrite files as there is no counter-option!

When favorites are available, they can be generated simply by invoking:

```cli
cargo gen <favorite>
```

or slightly more involved:

```cli
cargo generate demo --branch mybranch --name expanded_demo --subfolder myfolder
```

> ⚠️ NOTE: when `<favorite>` is not defined in the config file, it is interpreted as a git repo like as if `--git <favorite>`
