## Rhai extensions

Besides the basic [`Rhai`] features, these are the modules/behaviors defined:

### Variables with the `variable` module

#### get/set

* **`variable::is_set(name: &str) -> bool`**

  Returns true if the variable/placeholder has been set for the template

* **`variable::get(name: &str) -> value`**

  Gets any defined variable in the `Liquid` template object

* **`variable::set(name: &str, value: (&str|bool))`**

  Set new or overwrite existing variables. Do not allow to change types.
  Note that you can set entire arrays with this (e.g. `variable::set("array",["a","b"])`) but not individual elements (`variable::set("array[1]","a")` will not work).

#### Prompt for values with `variable::prompt`

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

### Files with the `file` module

* **`file::exists(path: &str)`**

  Test if a path exists

* **`file::rename(from: &str, to: &str)`**

  Rename one of the files in the template folder

* **`file::delete(path: &str)`**

  Delete a file or folder inside the template folder

* **`file::write(file: &str, content: &str)`**

  Create/overwrite a file inside the template folder

* **`file::write(file: &str, content: Array)`**

  Create/overwrite a file inside the template folder, each entry in the array on a new line
  
* **`file::listdir(path = ".") -> Array<String>`**

  List the contents of a directory
  
  Note: The path is relative to the template folder, and cannot be outside the template folder.
  
  Examples:
  ```rhai
  let files = file::listdir();
  for f in files {
      print(`file: ${f}`);
  }
  
  // this is actually the same as above, the path must be inside the template directory, cannot be absolute or ourside
  let files = file::listdir(".");
  for f in files {
      print(`file: ${f}`);
  }
  ```
  
  See also: [the many-hooks-in-action example project](https://github.com/cargo-generate/cargo-generate/blob/main/example-templates/many-hooks-in-action/sed-license.rhai#L18)

### The `system` module

* **`system::command(cmd: &str, args: Array = []) -> value`**

  Execute a command on the system generating the project from a template.

  The user will be prompted with 

  ```
  The template is requesting to run the following command. Do you agree?
  <command> <args>
  ```

  unless the user uses the flag `--allow-commands`. If the user attempts to use the
  `--silent` flag without the `--allow-commands` flag will fail.
  
  Examples:
  ```rhai
  // this returns the PWD as a string
  let pwd = system::command("pwd");
  
  // but this works too and does the same
  system::command("pwd", []);
  
  // this will cat a file and returns the content
  let content = system::command("cat", ["file.txt"]);
  ```
  
  See also: [the many-hooks-in-action example project](https://github.com/cargo-generate/cargo-generate/blob/main/example-templates/many-hooks-in-action/sed-license.rhai#L11)

* **`system::date() -> Date`**
  
  Get the date in UTC from the system as an object with the properties `year`, `month`, and `day`.
  
### The `env` module

The `env` module provides access to environment variables.

* **`env::working_directory`**
  
  Returns the current working directory as a string. This is the directory where the `cargo-generate` pre-processes the template, before it is copied over to the users `destination` directory.
  
  Examples:
  ```rhai
  let wd = env::working_directory;
  print(`Working directory: ${wd}`);
  ```
  
  See also: [the many-hooks-in-action example project](https://github.com/cargo-generate/cargo-generate/blob/main/example-templates/many-hooks-in-action/post-script.rhai#L6)
  
* **`env::destination_directory`**
  
  Returns the destination directory as a string. This is the directory where the template is copied to, and where the user will find the generated project.
  
  Examples:
  ```rhai
  let dd = env::destination_directory;
  print(`Destination directory: ${dd}`);
  ```
  
### Other

* **`abort(reason: &str)`**: Aborts `cargo-generate` with a script error.

#### Changing case of strings

* **`to_kebab_case(str: &str) -> String`**

  `"We are going to inherit the earth."` => `"we-are-going-to-inherit-the-earth"`

* **`to_lower_camel_case(str: &str) -> String`**

  `"It is we who built these palaces and cities."` => `"itIsWeWhoBuiltThesePalacesAndCities"`

* **`to_pascal_case(str: &str) -> String`**

  Same as `to_upper_camel_case(str: &str) -> String`

* **`to_shouty_kebab_case(str: &str) -> String`**

  `"We are going to inherit the earth."` => `"WE-ARE-GOING-TO-INHERIT-THE-EARTH"`

* **`to_shouty_snake_case(str: &str) -> String`**

  `"That world is growing in this minute."` => `"THAT_WORLD_IS_GROWING_IN_THIS_MINUTE"`

* **`to_snake_case(str: &str) -> String`**

  `"We carry a new world here, in our hearts."` => `"we_carry_a_new_world_here_in_our_hearts"`

* **`to_title_case(str: &str) -> String`**

  `"We have always lived in slums and holes in the wall."` => `"We Have Always Lived In Slums And Holes In The Wall"`

* **`to_upper_camel_case(str: &str) -> String`**

  `"We are not in the least afraid of ruins."` => `"WeAreNotInTheLeastAfraidOfRuins"`


[`Rhai`]: https://rhai.rs/book/
