# Installation

## Using `cargo-generate` with vendored `libgit2` and system's OpenSSL
By default, cargo-generate uses vendored sources for `libgit2` and OpenSSL that is installed on your system.

```sh
cargo install cargo-generate
```

This requires the following dependencies on your system:
- `libssl-dev` (this could also be named openssl)

## Using `cargo-generate` with vendored OpenSSL
However, you can also opt in to use a vendored OpenSSL version.
So that you don't have to have OpenSSL installed and built it on the spot.

this would require the following dependencies on your system, as documented by the [`openssl` crate]:
- A C compiler (`gcc`, for example)
- `perl` and `perl-core`
- `make`

```sh
cargo install cargo-generate --features vendored-openssl
```

## Using `cargo-generate` with system's `libgit2` and system's OpenSSL
You can opt-out of vendored libraries and use `libgit2` and OpenSSL from your system
by building cargo-generate without the default dependencies.

```sh
cargo install cargo-generate --no-default-features
```

This will require the following dependencies on your system:
- `pkg-config`
- `libgit2`
- `libssl-dev` (this could also be named openssl)

## Using `pacman` (Arch Linux)

`cargo-generate` can be installed from the [community repository] for Arch Linux:

```sh
pacman -S cargo-generate
```

## Manual Installation

1. Download the binary tarball for your platform from our [releases page].
2. Unpack the tarball and place the binary `cargo-generate` in `~/.cargo/bin/`

[`openssl` crate]: https://docs.rs/openssl
[community repository]: https://archlinux.org/packages/community/x86_64/cargo-generate/
[releases page]: https://github.com/cargo-generate/cargo-generate/releases
