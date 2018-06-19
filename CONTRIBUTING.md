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
