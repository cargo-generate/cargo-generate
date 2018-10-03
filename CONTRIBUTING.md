# Contributing

## Filing an Issue

If you are trying to use `cargo-generate` and run into an issue- please file an
issue! We'd love to get you up and running, even if the issue you have might
not be directly related to the code in `cargo-generate`. This tool seeks to make
getting a project in Rust easy to start, so any sorts of feedback on usability,
usecases, or other aspects of this general problem space are welcome!

When filing an issue, do your best to be as specific as possible. Include
the version of cargo you are using (`cargo --version`), the version of Rust
you are using (`rustc --version`) and your operating system and version. The
faster was can reproduce your issue, the faster we can fix it for you!

## Testing your code

After writing your patch, or finishing an awesome feature, make sure that your
code does not collides with the existing code in executing the unit tests.

To execute the unit tests, please to run `cargo test` at the
project root.

For complex issues, and solutions, we did not created unit tests yet.
For example, to test if your code does not collides with the solution of the
[issue #83], you have to run those tests locally:

**1. Clone an existing template, without any git submodule, locally**

You can take one at the [Templates page].
For example:

```sh
cargo generate --git https://github.com/rustwasm/wasm-pack-template
```

Once you tested the project has been correctly cloned and setted, please to
do the same with a template that contains git submodules.

**2. Clone an existing template, with at least one git submodule, locally**

For example:

```sh
cargo generate --git https://github.com/k0pernicus/cargo-template-test-submodule
```

Please check that the project has been correctly cloned and setted, and check
if the repository contains initialized submodules.

**Your ideas/contributions are welcome to create automated tests for this** :)

## Submitting a PR

If you are considering filing a pull request, make sure that there's an issue
filed for the work you'd like to do. There might be some discussion required!
Filing an issue first will help ensure that the work you put into your pull
request will get merged :)

Before you submit your pull request, check that you have completed all of the
steps mentioned in the pull request template. Link the issue that your pull
request is responding to, and format your code using [rustfmt][rustfmt].

### Configuring rustfmt

Before submitting code in a PR, make sure that you have formatted the codebase
using [rustfmt][rustfmt]. `rustfmt` is a tool for formatting Rust code, which
helps keep style consistent across the project. If you have not used `rustfmt`
before, here's how to get setup:

**1. Use Stable Toolchain**

Use the `rustup override` command to make sure that you are using the stable
toolchain. Run this command in the `cargo-generate` directory you cloned.

```sh
rustup override set stable
```

**2. Add the rustfmt component**

Install the most recent version of `rustfmt` using this command:

```sh
rustup component add rustfmt-preview --toolchain stable
```

**3. Running rustfmt**

To run `rustfmt`, use this command:

```sh
cargo +stable fmt
```

[rustfmt]: https://github.com/rust-lang-nursery/rustfmt

## Conduct

We expect everyone who participates in this project in anyway to be friendly,
open-minded, and humble. We have a [Code of Conduct], and expect you to have
read it. If you have any questions or concerns, feel free to reach out to
Ashley Williams, ashley666ashley@gmail.com.

[Code of Conduct]: CODE_OF_CONDUCT.md
[issue #83]: https://github.com/ashleygwilliams/cargo-generate/issues/83
[Templates page]: TEMPLATES.md
