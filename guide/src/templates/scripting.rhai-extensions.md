## Rhai extensions

Besides the basic [`Rhai`] features, these are the modules/behaviors defined:

### Variables

#### get/set

* **`variable::is_set(name: &str) -> bool`**

  Returns true if the variable/placeholder has been set for the template

* **`variable::get(name: &str) -> value`**

  Gets any defined variable in the `Liquid` template object

* **`variable::set(name: &str, value: (&str|bool))`**

  Set new or overwrite existing variables. Do not allow to change types.

#### Prompt

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

### Files

* **`file::exists(path: &str)`**

  Test if a path exists

* **`file::rename(from: &str, to: &str)`**

  Rename one of the files in the template folder

* **`file::delete(path: &str)`**

  Delete a file or folder inside the template folder

* **`file::write(file: &str, content: &str)`**

  Create/overwrite a file inside the template folder

* **`file::write(file: &str, content: Array)`*

  Create/overwrite a file inside the template folder, each entry in the array on a new line

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
