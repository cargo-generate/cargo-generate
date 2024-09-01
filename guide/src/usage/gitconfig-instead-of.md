# `.gitconfig` and `insteadOf` configuration

⚠️ New in version [0.22.0]

git supports a magic trick to rewrite urls on the fly. This is done by adding a `url.<base>.insteadOf` configuration to your `.gitconfig` file.

In cargo-generate this is supported as well.

For example, if you prefer the ssh over the https urls and you want to use `cargo-generate` with it, you can add the following to your `.gitconfig`:

```.gitconfig
# ~/.gitconfig

[url "git@github.com:"]
insteadOf = https://github.com/
```

and then you can use `cargo-generate` with the `https` url:

```sh
RUST_LOG=debug cargo generate https://github.com/Rahix/avr-hal-template.git

🔧   gitconfig 'insteadOf' lead to this url: git@github.com:Rahix/avr-hal-template.git

...
```

In this case please notice the ssh url is `git@github.com:` if you prefer the more explicit notation you can also write it like this:

```.gitconfig
# ~/.gitconfig

[url "ssh://git@github.com/"]
insteadOf = https://github.com/
```

that would lead to the same result, with slightly different url:

```sh
RUST_LOG=debug cargo generate https://github.com/Rahix/avr-hal-template.git

🔧   gitconfig 'insteadOf' lead to this url: ssh://git@github.com/Rahix/avr-hal-template.git

...
```

> ⚠️ NOTE: `RUST_LOG=debug` would allow you to see the rewritten url in the output.

In cases where you have a different `.gitconfig` location, you can use the `--gitconfig` argument to specify the path to the `.gitconfig` file, like this:

```sh
$ cd /path/to/my/workspace
$ cat .gitconfig
[url "git@github.com:"]
insteadOf = https://github.com/

$ cargo generate --gitconfig ./.gitconfig https://github.com/Rahix/avr-hal-template.git
```

[0.22.0]: https://github.com/cargo-generate/cargo-generate/releases/tag/v0.22.0
