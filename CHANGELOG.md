# Changelog

## 💥 0.1.1

  - ### 🤕 Fixes

    - **Fix command to work properly as a cargo command - [csmoe], [issue/39] [pull/44]**

      Previous to this commit, `cargo-generate` was a CLI tool that was invoked by the
      command `cargo-generate` (note the dash). However, this tool intends to be a cargo
      subcommand! This commit changes how you invoke the tool- no more dash!

      ```
      cargo generate --git https://github.com/username/project --name look-ma-no-dash
      ```

      [csmoe]: https://github.com/csmoe
      [issue/39]: https://github.com/ashleygwilliams/cargo-generate/issues/39
      [pull/44]: https://github.com/ashleygwilliams/cargo-generate/pull/44

    - **Fix casing on `crate_name` substitution - [ashleygwilliams], [issue/41] [pull/56]**

      `crate_name` substitution is supposed to be a convenience, converting the given
      project's name to a name that you could use with `extern crate` or in other *in-code*
      situations. Just one problem- before this commit, it didn't change the case! 
      Now it will. Thanks so much to [fitzgen] for filing this issue (and a bunch of others)!

      [ashleygwilliams]: https://github.com/ashleygwilliams
      [issue/41]: https://github.com/ashleygwilliams/cargo-generate/issues/41
      [pull/56]: https://github.com/ashleygwilliams/cargo-generate/pull/56
      [fitzgen]: https://github.com/fitzgen

  - ### 📖 Documentation

    - **Document build and runtime dependencies - [migerh], [issue/42] [pull/45]**

      There are a few dependencies for the project that we hadn't documented. Many folks
      have these already installed, but some don't- so it's great that they are now well
      documented in the `README`.

      [migerh]: https://github.com/migerh
      [issue/42]: https://github.com/ashleygwilliams/cargo-generate/issues/42
      [pull/45]: https://github.com/ashleygwilliams/cargo-generate/pull/45

    - **Update README and demo.gif to address The Dash - [ashleygwilliams], [pull/60]**

      [pull/60]: https://github.com/ashleygwilliams/cargo-generate/pull/60

    - **Typo Fix - [rahulthakoor], [pull/36]**

      [rahulthakoor]: https://github.com/rahul-thakoor
      [pull/36]: https://github.com/ashleygwilliams/cargo-generate/pull/36

  - ### 🛠️ Maintenance

    - **Test substitutions - [ashleygwilliams], [issue/34] [pull/56]**

      We had features we weren't testing. This PR now adds test coverage for:

        - substitution of `crate_name`
        - correct casing change of `crate_name`
        - substitution in files beyond `Cargo.toml`

      We still don't have full coverage but at least it's improving!

      [issue/34]: https://github.com/ashleygwilliams/cargo-generate/issues/34

    - **Split test helpers into files - [ashleygwilliams], [issue/33] [pull/35]**

      "i like small files and i cannot lie"

      [ashleygwilliams]: https://github.com/ashleygwilliams
      [issue/33]: https://github.com/ashleygwilliams/cargo-generate/issues/33
      [pull/35]: https://github.com/ashleygwilliams/cargo-generate/pull/35

## 🌌 0.1.0

- First release! 
