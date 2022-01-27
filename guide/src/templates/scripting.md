# Pre/Post Scripts

In `cargo-generate.toml` write a `[hooks]` section, example:

```toml
[template]
cargo_generate_version = "0.10.0"

[hooks]
pre = ["pre-script.rhai"]
#post = [...]

[placeholders]
license = { type = "string", prompt = "What license to use?", choices = ["APACHE", "MIT"], default = "MIT" }
```

Now, write the script in [`Rhai`], utilizing the `cargo-generate` [provided extensions](#Rhai-extensions):

```rhai
// we can see existing variables.
// note that template and Rhai variables are separate!
let crate_type = variable::get("crate_type")
debug(`crate_type: ${crate_type}`);

let license = variable::get("license").to_upper();
while switch license {
  "APACHE" => {
    file::delete("LICENSE-MIT");
    file::rename("LICENSE-APACHE", "LICENSE");
    false
  }
  "MIT" => {
    file::delete("LICENSE-APACHE");
    file::rename("LICENSE-MIT", "LICENSE");
    false
  }
  _ => true,
} {
  license = variable::prompt("Select license?", "MIT", [
    "APACHE",
    "MIT",
  ]);
}
variable::set("license", license);
```

### Rhai extensions

Besides the basic [`Rhai`] features, these are the modules/behaviors defined:

#### Variables

##### get/set

* **`variable::is_set(name: &str) -> bool`**

  Returns true if the variable/placeholder has been set for the template

* **`variable::get(name: &str) -> value`**

  Gets any defined variable in the `Liquid` template object

* **`variable::set(name: &str, value: (&str|bool))`**

  Set new or overwrite existing variables. Do not allow to change types.

##### Prompt

* **`variable::prompt(text: &str, default_value: bool) -> value`**

  Prompt the user for a boolean value

* **`variable::prompt(text: &str) -> value`**

  Prompt the user for a string value

* **`variable::prompt(text: &str, default_value: &str) -> value`**

  Prompt the user for a string value, with a default already in place

* **`variable::prompt(text: &str, default_value: &str, regex: &str) -> value`**

  Prompt the user for a string value, validated with a regex

* **`variable::prompt(text: &str, default_value: &str, choices: Array) -> value`**

  Prompt the user for a choice value

#### Files

* **`file::rename(from: &str, to: &str)`**

  Rename one of the files in the template folder

* **`file::delete(path: &str)`**

  Delete a file or folder inside the template folder

* **`file::write(file: &str, content: &str)`**

  Create/overwrite a file inside the template folder

* **`file::write(file: &str, content: Array)`*

  Create/overwrite a file inside the template folder, each entry in the array on a new line

#### Other

* **abort(reason: &str)**: Aborts `cargo-generate` with a script error.
