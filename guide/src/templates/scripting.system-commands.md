# System commands

Hook scripts can run programs with `system::command`. First, register the script
as a hook in `cargo-generate.toml`:

```toml
[hooks]
post = ["post-script.rhai"]
```

Then call the program from `post-script.rhai`, passing its arguments in an
array:

```rhai
system::command("cargo", ["fmt"]);

let rustc_version = system::command("rustc", ["--version"]);
print(`Generated with ${rustc_version}`);
```

Commands run in the template's working directory through `sh` on Unix and
`cmd` on Windows, so available programs and shell syntax can vary by platform.
On success, `system::command` returns trimmed standard output, or `()` if the
command produced no output. A command that cannot start or exits unsuccessfully
stops template generation with an error.

By default, `cargo-generate` shows the requested command and asks the user to
approve it. To run commands without confirmation, the user must opt in:

```console
cargo generate --git https://github.com/example/template.git --allow-commands
```

The `--silent` option cannot run a command hook unless `--allow-commands` is
also set. A template cannot enable this permission for itself.

> **Security warning:** `--allow-commands` lets a template execute arbitrary
> shell commands with your user account's permissions and without further
> confirmation. Those commands can access secrets, modify files outside the
> generated project, or communicate over the network. Arguments are joined
> into shell command text without automatic quoting or escaping, so template
> authors must not insert untrusted values. Only enable commands for templates
> you trust after reviewing their hook scripts and imported modules.

See the [`system` module reference](scripting.rhai-extensions.md#the-system-module)
for the complete function signature and more examples.
