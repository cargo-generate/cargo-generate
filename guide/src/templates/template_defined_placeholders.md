# Template Defined Placeholders

Template defined placeholders offer a powerful way for template authors to customize project templates and streamline project creation. In addition to defining placeholders directly within template files, users can also define placeholders in the `cargo-generate.toml` file, providing additional flexibility and customization options.

## Defining Placeholders in `cargo-generate.toml`

To define placeholders in the `cargo-generate.toml` file, template authors can specify them under the `placeholders` section using the following syntax:

```toml
[placeholders]
placeholder_name = { prompt = "Enter your name", choices = ["Alice", "Bob"], default = "Alice", type = "string" }
```

- `placeholder_name`: The name of the placeholder.
- `prompt`: The prompt message displayed to the user during project creation.
- `choices` (optional): A list of predefined choices for the placeholder value.
- `default` (optional): The default value for the placeholder if no user input is provided.
- `regex` (optional and only for string-like types): The entered value is validated against this regex.
- `type`: The data type of the placeholder value (see [Supported Types](#supported-types)).

## Prompt, Choices, and Default Values

- **Prompt**: With the `prompt` will be displayed it to the user during project creation, prompting them to provide a value for the placeholder.
- **Choices**: If `choices` are specified, `cargo-generate` will present them as options to the user, restricting the input to the predefined choices and provide more convenience.
- **Default Value**: If a `default` value is provided and the user does not provide input, `cargo-generate` will use the default value for the placeholder.

## Supported Types

`cargo-generate` supports the following placeholder value types:

- `"string"`: Represents a string value.
- `"text"`: Represents a multiline string value. (terminated by hitting CTRL-D)
- `"editor"`: Represents a multiline string value, collected from the user by a real terminal editor.
- `"bool"`: Represents a boolean value (`true` or `false`).

## Example

Consider the following `cargo-generate.toml` file:

```toml
[placeholders]
project_name = { prompt = "Enter project name", default = "my_project", type = "string" }
environment = { prompt = "Which environment?", choices = ["dev", "prod"], default = "dev", type = "string"}
use_git = { prompt = "Initialize Git repository?", default = true, type = "bool" }
phone_number = { prompt = "What's your phone number?", type = "string", regex = "^[0-9]+$" }
```

During project creation, `cargo-generate` will prompt the user to provide values for `project_name`, `use_git` and `phone_number` using the specified prompts, choices, and default values.

Further `phone_number` is validated against the provided regex, hence it can only contain digits.

### Conclusion

Template defined placeholders, defined in the `cargo-generate.toml` configuration file, offer powerful customization options for project templates. By specifying prompts, choices, default values, and supported types, template authors can create intuitive and flexible project scaffolding experiences, enhancing developer productivity and project consistency.

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

## Further examples

You can find further examples in the [example-templates folder](./example-templates/) you will find further examples that provide some template provided placeholders.
