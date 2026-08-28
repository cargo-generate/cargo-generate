# Installation

## From crates.io

```sh
cargo install cargo-generate --locked
```

No system libraries are required. New in version [1.0.0] is the pure-Rust git
stack via [`gix`] and TLS via [`rustls`], so a plain Rust toolchain is all you
need — no `pkg-config`, `libgit2`, `libssl-dev`, `perl`, or C compiler.

## Using `pacman` (Arch Linux)

`cargo-generate` can be installed from the [extra repository] for Arch Linux:

```sh
pacman -S cargo-generate
```

## Manual Installation

1. Download the binary tarball for your platform from our [releases page].
2. Unpack the tarball and place the binary `cargo-generate` in `~/.cargo/bin/`.

[1.0.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v1.0.0
[`gix`]: https://crates.io/crates/gix
[`rustls`]: https://crates.io/crates/rustls
[extra repository]: https://archlinux.org/packages/extra/x86_64/cargo-generate/
[releases page]: https://github.com/cargo-generate/cargo-generate/releases
