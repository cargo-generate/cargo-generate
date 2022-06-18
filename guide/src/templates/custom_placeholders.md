# Template defined placeholders

Sometimes templates need to make decisions. For example one might want to conditionally include some code or not.
Another use case might be that the user of a template should be able to choose out of provided options in an interactive way.
Also, it might be helpful to offer a reasonable default value that the user just simply can use.

Since version [0.6.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.6.0) it is possible to use placeholders in a `cargo-generate.toml` that is in the root folder of a template.
Here [an example](https://github.com/sassman/hermit-template-rs):

```toml
[placeholders.hypervisor]
type = "string"
prompt = "What hypervisor to use?"
choices = ["uhyve", "qemu"]
default = "qemu"

[placeholders.network_enabled]
type = "bool"
prompt = "Want to enable network?"
default = true
```

As you can see the `placeholders` configuration section accepts a table of keywords that will become the placeholder name.

In this example the placeholder `hypervisor` and `network_enabled` will become template variables and can be used like this:

```rs
{% if network_enabled %}
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    loop {
        let (conn, addr) = listener.accept().unwrap();
        println!("Incoming Connection from {}", addr);
        std::io::copy(&mut &conn, &mut &conn).unwrap();
    }
}
{% else %}
fn main() {
    println!("Hello Rusty Hermit 🦀");
}
{% endif %}
```

> 💡 Tip: similar to `dependencies` in the `Cargo.toml` file you can also list them as one liners:

```toml
[placeholders]
hypervisor = { type = "string", prompt = "What hypervisor to use?", choices = ["uhyve", "qemu"], default = "qemu" }
network_enabled = { type = "bool", prompt = "Want to enable network?", default = true }
```

### `prompt` property

The `prompt` will be used to display a question / message for this very placeholder on the interactive dialog when using the template.

```plain
🤷  What hypervisor to use? [uhyve, qemu] [default: qemu]:
```

### `type` property

A placeholder can be of type `string` or `bool`. Boolean types are usually helpful for conditionally behaviour in templates.

### `choices` property (optional)

A placeholder can come with a list of choices that the user can choose from.
It's further also validated at the time when a user generates a project from a template.

```toml
choices = ["uhyve", "qemu"]
```

### `default` property (optional)

A `default` property must mach the type (`string` | `bool`) and is optional. A default should be provided, to ease the interactive process.
As usual the user could press `enter` and the default value would simply be taken, it safes time and mental load.

```toml
default = 'qemu'
```

### `regex` property (optional)

A `regex` property is a string, that can be used to enforce a certain validation rule. The input dialog will keep repeating
until the user entered something that is allowed by this regex.

### Placeholder Examples

An example with a regex that allows only numbers

```toml
[placeholders]
phone_number = { type = "string", prompt = "What's your phone number?", regex = "^[0-9]+$" }
```

## Default values for placeholders

For automation purposes the user of the template may provide the values for the keys in the template using one or more of the following methods.

The methods are listed by falling priority.

### `--define` or `-d` flag

The user may specify variables individually using the `--define` flag.

```sh
cargo generate template-above -n project-name -d hypervisor=qemu -d network_enabled=true
```

### `--template_values_file` flag

The user of the template may provide a file containing the values for the keys in the template by using the `--template-values-file` flag.

> ⚠️ NOTE: A relative path will be relative to current working dir, which is *not* inside the expanding template!

The file should be a toml file containing the following (for the example template provided above):

```toml
[values]
hypervisor = "qemu"
network_enabled = true
```

#### Individual values via environment variables

Variables may be specified using environment variables. To do so, set the env var `CARGO_GENERATE_VALUE_<variable key>` to the desired value.

```sh
set CARGO_GENERATE_VALUE_HYPERVISOR=qemu
set CARGO_GENERATE_VALUE_NETWORK_ENABLED=true
cargo generate template-above
```

> ⚠️ Windows does not support mixed case environment variables. Internally, `cargo-generate` will ensure the variable name is all lowercase. For that reason, it is strongly recommended that template authors only use lowercase variable/placeholder names.

#### Template values file via environment variable

The user may use the environment variable `CARGO_GENERATE_TEMPLATE_VALUES` to specify a file with default values.

For the file format, see above.

#### Default values

Default values may be specified in the config file (specified with the `--config` flag, or in the default config file `$CARGO_HOME/cargo-generate`)

**Example config file:**

```toml
[values]
placeholder1 = "default value"

[favorites.my_favorite]
git = "https://github.com/username-on-github/mytemplate.git"

[favorites.my_favorite.values]
placeholder1 = "default value overriding the default"
placeholder2 = "default value for favorite"
```
