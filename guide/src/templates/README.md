# Templates

Templates are git repositories whose files contain placeholders. The current
supported placeholders are:

* `{{authors}}`

  this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration. It will either be on the form `username <email>` or just plain `username`.
* `{{project-name}}`

  this is supplied by either passing the `--name` flag to the command or working with the interactive CLI to supply a name.
* `{{crate_name}}`

  the snake_case_version of `project-name`
* `{{crate_type}}`

  this is supplied by either passing the `--bin` or `--lib` flag to the command line, contains either `bin` or `lib`, `--bin` is the default
* `{{os-arch}}`

  contains the current operating system and architecture ex: `linux-x86_64`
* `{{username}}`

  this will be filled in by a function borrowed from Cargo's source code, that determines your information from Cargo's configuration.

Additionally, **all filters and tags** of the liquid template language are supported.
For more information, check out the [Liquid Documentation on `Tags` and `Filters`][liquid].

[liquid]: https://shopify.github.io/liquid

You can use those placeholders in the file and directory names of the generated project.
For example, for a project named `awesome`, the filename `{{project_name}}/{{project_name}}.rs` will be transformed to `awesome/awesome.rs` during generation.
Only files that are **not** listed in the exclude settings will be templated.

> ⚠️ NOTE: invalid characters for a filename or directory name will be sanitized after template substitution. Invalid is e.g. `/` or `\`.

> ⚠️ **Deprecated** in favor of using [ignore in `cargo-generate.toml`](#Ignoring-files)
>
> You can also add a `.genignore` file to your template. The files listed in the `.genignore` file
> will be removed from the local machine when `cargo-generate` is run on the end user's machine.
> The `.genignore` file is always ignored, so there is no need to list it in the `.genignore` file.

## Templates by the community

It's encouraged to classify your template repository [with a GitHub topic](https://docs.github.com/en/github/administering-a-repository/managing-repository-settings/classifying-your-repository-with-topics) labeled `cargo-generate`.

So that every developer can find the template via [cargo-generate topic on GitHub](https://github.com/topics/cargo-generate).

If you have a great template, please tag your repository with the topic [and tweet about it](https://twitter.com/intent/tweet?text=See%20my%20new%20%23cargogenerate%20%23template%20%0A%0A%3E%20your%20link%20goes%20here) by including the hashtag [`#cargogenerate`](https://twitter.com/search?q=%23cargogenerate&src=typed_query) (since twitter does not support hashtags with `-`).

> ⚠️ Note: the list of [currently available templates](https://github.com/cargo-generate/cargo-generate/blob/main/TEMPLATES.md) is still available, but is now deprecated.

### Example for `--bin` and `--lib`

A template could be prepared in a way to act as a binary or a library. For example the `Cargo.toml` might look like:

```toml
[package]
# the usual stuff

[dependencies]
{% if crate_type == "bin" %}
structopt = "0.3.21"
{% endif %}
# other general dependencies

{% if crate_type == "bin" %}
[[bin]]
path = "src/main.rs"
name = "{{crate_name}}-cli"
{% endif %}
```

Now a user of this template could decide weather they want the binary version by passing `--bin`
or use only the library version by passing `--lib` as a command line argument.
