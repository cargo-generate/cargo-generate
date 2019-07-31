# Changelog

## 🛠 0.3.1

  - ### 🤕 Fixes

    - **Fix messages related to rename behavior - [xortive], [pull/162]**
        
        `--force` would stop `cargo-generate` renaming your project, but we would
        still message that the rename was happening. Not anymore!

        [xortive]: https://github.com/xortive
        [pull/162]: https://github.com/ashleygwilliams/cargo-generate/pull/162

    - **Use Vendored OpenSSL for macOS - [xortive], [pull/169]**
      
        Our prebuilt binaries for macOS were using dynamically linked OpenSSL, 
        and our CI was dynamically linking to a version of OpenSSL not installed
        by default on most macs. Now, the macOS release build of `cargo-generate`
        will work out of the box utilizing staticly linked, vendored OpenSSL.

        [xortive]: https://github.com/xortive
        [pull/169]: https://github.com/ashleygwilliams/cargo-generate/pull/169

  - ### 👯 New Templates

    - **`bluepill` stm32 microcontroller board template - [mendelt], [pull/156]**

        [mendelt]: https://github.com/mendelt
        [pull/156]: https://github.com/ashleygwilliams/cargo-generate/pull/156

    - **`cmdr` commandline appliction template - [mendelt], [pull/156]**

        [mendelt]: https://github.com/mendelt
        [pull/156]: https://github.com/ashleygwilliams/cargo-generate/pull/156

    - **`ggez` rust gamedev template - [cyclowns], [pull/167]**
    
        [cyclowns]: https://github.com/cyclowns
        [pull/167]: https://github.com/ashleygwilliams/cargo-generate/pull/167

- ### 🛠️ Maintenace

    - **Update to liquid 0.19 - [epage], [pull/165]**

        [epage]: https://github.com/epage
        [pull/165]: https://github.com/ashleygwilliams/cargo-generate/pull/165

## ⭐ 0.3.0

  - ### ✨ Features

    - **Support case filters in templates - [epage], [issue/117] [pull/140]**

        Because we leverage `liquid` as a templating engine under the hood, we
        can add some specific filters to our substitutions to add a little more
        awesome. This feature adds 4 filters:

        - `capitalize`
        - `pascal_case`
        - `kebab_case` 
        - `snake_case`

        Now we can tame any set of letters with any type of capitalization or
        dash! Give them a try and let us know what other types of features you'd
        like to see.

        [issue/117]: https://github.com/ashleygwilliams/cargo-generate/issues/117
        [pull/140]: https://github.com/ashleygwilliams/cargo-generate/pull/140

  - ### 🤕 Fixes

    - **Windows releases on Appveyor - [jaysonsantos], [issue/145] [pull/146]**

        Thanks to a new tool, [`wrangler`], that uses `cargo-generate` as a
        dependency, it was discovered that we were shipping broken Windows
        binaries. We've fixed that now!

        [`wrangler`]: https://github.com/cloudflare/wrangler
        [jaysonsantos]: https://github.com/jaysonsantos
        [issue/145]: https://github.com/ashleygwilliams/cargo-generate/issues/145
        [pull/146]: https://github.com/ashleygwilliams/cargo-generate/pull/146

  - ### 👯 New Templates

    - **`procmacro-quickstart` template - [eupn], [pull/141]**

        [eupn]: https://github.com/eupn
        [pull/141]: https://github.com/ashleygwilliams/cargo-generate/pull/141

  - ### 🛠️ Maintenance

    - **Update to 2018 Edition - [ashleygwilliams], [issue/131] [pull/147]**

        This was a fun one and additionally involved upgrading us to `quicli` 0.4!
        Welcome to 2018, `cargo-generate`.

        [issue/131]: https://github.com/ashleygwilliams/cargo-generate/issues/131
        [pull/147]: https://github.com/ashleygwilliams/cargo-generate/pull/147

    - **Shallow `main` refactor - [DD5HT], [pull/115]**

        [pull/115]: https://github.com/ashleygwilliams/cargo-generate/pull/115

    - **Update `liquid` - [epage], [pull/139]**

        [epage]: https://github.com/epage
        [pull/139]: https://github.com/ashleygwilliams/cargo-generate/pull/139

## 🌟 0.2.2

  - ### 🤕 Fixes

    - **fix relative paths to templates - [DD5HT], [issue/128] [pull/129]**

      When we previously merged the PR in 0.2.0 that leveraged `cargo` to clone the templates,
      enabling folks to work with private repositories- we introduced a `GitConfig::new` function
      (replacing work done previously by `libgit2`). This function works great- but did not support
      relative paths. We didn't catch this because we weren't testing the relative paths usecase!

      With this PR, [DD5HT] has restored the relative path functionality- and added a test to prevent
      further regressions of this function!

      [issue/128]: https://github.com/ashleygwilliams/cargo-generate/issues/128
      [pull/129]: https://github.com/ashleygwilliams/cargo-generate/pull/129

  - ### 🛠️ Maintenance

    - **cargo update/cargo fmt - [ashleygwilliams], [pull/134] [pull/133]**

      [pull/133]: https://github.com/ashleygwilliams/cargo-generate/pull/133
      [pull/134]: https://github.com/ashleygwilliams/cargo-generate/pull/134

## 🌠 0.2.1

  - ### 🤕 Fixes

    - **don't error on missing `.genignore` file - [DD5HT], [issue/116] [pull/120]**

      With 0.2.0 we introduced the idea of a `.genignore` file, however, we didn't account
      the situation where one would not be present. Thanks for filing [an issue][issue/116]
      [Diggsey] and thanks for the quick fix [DD5HT]!

      [issue/116]: https://github.com/ashleygwilliams/cargo-generate/issues/116
      [pull/120]: https://github.com/ashleygwilliams/cargo-generate/pull/120
      [Diggsey]: https://github.com/Diggsey

    - **enable use on private repositories- [ChristopherMacGown], [pull/119]**

      This PR leveraged `cargo` to enable the ability to pull authenticated repos. As this
      project grows into something we'd like to integrate into `cargo`, this gives us
      greater functionality and also moves us closer to `cargo`'s codebase. Yay!

      [ChristopherMacGown]: https://github.com/ChristopherMacGown
      [pull/119]: https://github.com/ashleygwilliams/cargo-generate/pull/119

    - **exclude submodules git metadata - [ChristopherMacGown], [pull/119]**
  
      Two bugs, one PR! Adding a test found that git metadata wasn't being excluded from
      submodules- now it is! Thanks so much!

  - ### 👯 New Templates

    - **`actix-tera` template - [antweiss], [pull/123]**
    - **`samp rust sdk` template - [Sreyas-Sreelal], [pull/121]**

    [antweiss]: https://github.com/antweiss
    [pull/123]: https://github.com/ashleygwilliams/cargo-generate/pull/123
    [Sreyas-Sreelal]: https://github.com/Sreyas-Sreelal
    [pull/121]: https://github.com/ashleygwilliams/cargo-generate/pull/121

## 💫 0.2.0

  - ### ✨ Features

    - **Support templates that use git submodules - [k0pernicus], [issue/83] [pull/104]**

      We now support templates that use git submodules! Yay!

      [k0pernicus]: https://github.com/k0pernicus
      [issue/83]: https://github.com/ashleygwilliams/cargo-generate/issues/83
      [pull/104]: https://github.com/ashleygwilliams/cargo-generate/pull/104

    - **Binary Releases for Linux, MacOS, and Windows - [ashleygwilliams], [issue/99] [pull/111] [pull/112]**

      Motivated by a desire to more easily distributed the project - we now build binaries
      for our releases. No more waiting for compilation! You can just download and go!

      [issue/99]: https://github.com/ashleygwilliams/cargo-generate/issues/99
      [pull/111]: https://github.com/ashleygwilliams/cargo-generate/pull/111
      [pull/112]: https://github.com/ashleygwilliams/cargo-generate/pull/112

    - **Allow Liquid Templating `date` filter - [ashleygwilliams], [issue/70] [pull/106]**

      By request, we've turned on the `date` filter for our templates. Now you can add
      nicely formatted dates to your projects! For more information, check out the
      [Liquid `date` filter documentation].

      [Liquid `date` filter documentation]: https://shopify.github.io/liquid/filters/date/
      [issue/70]: https://github.com/ashleygwilliams/cargo-generate/issues/70
      [pull/106]: https://github.com/ashleygwilliams/cargo-generate/pull/106

    - **Add `.genignore`, ability to ignore files - [DD5HT], [issue/82] [pull/96]**

      You can now add a `.genignore` file to your template. This file will specify the files
      to be "cleaned up" or "removed" from the template once it has been cloned to the user's
      local machine.

      [issue/82]: https://github.com/ashleygwilliams/cargo-generate/issues/82
      [pull/96]: https://github.com/ashleygwilliams/cargo-generate/pull/96

    - **Add `--branch` for specifying a branch - [posborne], [issue/71] [pull/94]**

      We originally had no way to specify a git template on a per branch basis, opting to
      only support the primary branch. Now you can specify a branch:

      ```
      cargo generate --git <gitURL> --branch <branchname>
      ```

      [posborne]: https://github.com/posborne
      [issue/71]: https://github.com/ashleygwilliams/cargo-generate/issues/71
      [pull/94]: https://github.com/ashleygwilliams/cargo-generate/pull/94

    - **Warn user if we change project name casing - [k0pernicus], [issue/65] [pull/84]**

      `cargo-generate` will automagically "fix" the casing of your project name to
      match Cargo standards. If we end up changing the name you provide- we'll warn
      to let you know!

      [k0pernicus]: https://github.com/k0pernicus
      [issue/65]: https://github.com/ashleygwilliams/cargo-generate/issues/65
      [pull/84]: https://github.com/ashleygwilliams/cargo-generate/pull/84

    - **Add `--force` flag to skip casing check on project name - [toVersus], [issue/66] [pull/69]**

      `cargo-generate` will automagically "fix" the casing of your project name to 
      match Cargo standards. If you'd like to skip that, you can add `--force`.

      [toVersus]: https://github.com/toVersus
      [issue/66]: https://github.com/ashleygwilliams/cargo-generate/issues/66
      [pull/69]: https://github.com/ashleygwilliams/cargo-generate/pull/69

    - **Add short flag `-n` for `--name` - [DD5HT], [issue/73] [pull/77]**

      [issue/73]: https://github.com/ashleygwilliams/cargo-generate/issues/73
      [pull/77]: https://github.com/ashleygwilliams/cargo-generate/pull/77

    - **List of available templates - [ashleygwilliams], [issue/74] [issue/50] [pull/75]**

      We are now keeping a running list of templates that are available to use with
      `cargo-generate`. Please add more!

      [issue/74]: https://github.com/ashleygwilliams/cargo-generate/issues/74
      [issue/50]: https://github.com/ashleygwilliams/cargo-generate/issues/50
      [pull/75]: https://github.com/ashleygwilliams/cargo-generate/pull/75

    - **Add short command `cargo gen` - [DD5HT], [issue/53] [pull/72]**

      You can now use `cargo gen` as a short command for `cargo generate`.

      [DD5HT]: https://github.com/DD5HT
      [issue/53]: https://github.com/ashleygwilliams/cargo-generate/issues/53
      [pull/72]: https://github.com/ashleygwilliams/cargo-generate/pull/72

  - ### 🛠️ Maintenance

    - **Fixed some clippy warnings - [4tm4j33tk4ur], [pull/109]**

      [4tm4j33tk4ur]: https://github.com/4tm4j33tk4ur
      [pull/109]: https://github.com/ashleygwilliams/cargo-generate/pull/109

    - **Test safety of `.genignore` - [ashleygwilliams], [issue/97] [pull/98]**

      [issue/97]: https://github.com/ashleygwilliams/cargo-generate/issues/97
      [pull/98]: https://github.com/ashleygwilliams/cargo-generate/pull/98

    - **`cargo update` and update `cargo fmt` call - [ashleygwilliams], [issue/86] [pull/88]**

      [issue/86]: https://github.com/ashleygwilliams/cargo-generate/issues/86
      [pull/88]: https://github.com/ashleygwilliams/cargo-generate/pull/88

    - **Test project name casing - [ashleygwilliams], [issue/63] [pull/64]**

      [issue/63]: https://github.com/ashleygwilliams/cargo-generate/issues/63
      [pull/64]: https://github.com/ashleygwilliams/cargo-generate/pull/64

    - **Move from `ident_case` to `heck` - [csmoe], [issue/57] [pull/62]**

      [issue/57]: https://github.com/ashleygwilliams/cargo-generate/issues/57
      [pull/62]: https://github.com/ashleygwilliams/cargo-generate/pull/62

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
