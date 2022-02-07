# Installation

## Using `cargo` with system's OpenSSL

```sh
cargo install cargo-generate
```

See the [`openssl-sys` crate readme] on how to obtain the OpenSSL library for your system. Alternatively, use the `vendored-openssl` flag if you do not want to install OpenSSL.

## Using `cargo` with vendored OpenSSL

> ⚠️ NOTE: `vendored-openssl` requires the following packages to be installed:
> - libssl-dev
> - gcc
> - m4
> - ca-certificates
> - make
> - perl

```sh
cargo install cargo-generate --features vendored-openssl
```

## Using `pacman` (Arch Linux)

`cargo-generate` can be installed from the [community repository] for Arch Linux:

```sh
pacman -S cargo-generate
```

## Manual Installation

1. Download the binary tarball for your platform from our [releases page].
2. Unpack the tarball and place the binary `cargo-generate` in `~/.cargo/bin/`

[`openssl-sys` crate readme]: https://crates.io/crates/openssl-sys
[community repository]: https://archlinux.org/packages/community/x86_64/cargo-generate/
[releases page]: https://github.com/cargo-generate/cargo-generate/releases
