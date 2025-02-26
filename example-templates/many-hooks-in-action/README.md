# Hooks template

This is an example template that uses hook scripts to let the user select a license file.

The selected license file will be renamed into `LICENSE`, and besides this `README.md` be the only files left after expansion.

## Expansion

```sh
cargo generate --name my-expanded-template gh:cargo-generate/cargo-generate example-templates/many-hooks-in-action
```

or to select the license directly from commandline:

```sh
cargo generate --name my-expanded-template gh:cargo-generate/cargo-generate example-templates/many-hooks-in-action -d license=mit
```
