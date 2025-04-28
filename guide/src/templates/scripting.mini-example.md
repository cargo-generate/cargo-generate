## Mini Example

In `cargo-generate.toml` write a `[hooks]` section:

```toml
[template]
cargo_generate_version = "0.10.0"

[hooks]
#init = [...]
pre = ["pre-script.rhai"]
#post = [...]

[placeholders]
license = { type = "string", prompt = "What license to use?", choices = ["APACHE", "MIT"], default = "MIT" }
```

Now, write the script in [`Rhai`], utilizing the `cargo-generate` [provided extensions](#Rhai-extensions):

```rhai
// we can see existing variables.
// note that template and Rhai variables are separate!
let crate_type = variable::get("crate_type");
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


[`Rhai`]: https://rhai.rs/book/
