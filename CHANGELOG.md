# Changelog

## [Unreleased](https://github.com/cargo-generate/cargo-generate/compare/0.20.0...HEAD)

## [0.20.0] 2024-02-17

[0.20.0]: https://github.com/cargo-generate/cargo-generate/compare/0.19.0...0.20.0

### ✨ Features

- Add some more tests regarding conditionals see [#1119](https://github.com/cargo-generate/cargo-generate/pull/1119) ([#1133](https://github.com/cargo-generate/cargo-generate/pull/1133))

### 📖 Documentation

- Add docs about the new feature ([#1074](https://github.com/cargo-generate/cargo-generate/pull/1074)) ([#1132](https://github.com/cargo-generate/cargo-generate/pull/1132))

### 🛠️  Maintenance

- Bump home from 0.5.5 to 0.5.9 ([#1085](https://github.com/cargo-generate/cargo-generate/pull/1085))
- Bump predicates from 3.0.4 to 3.1.0 ([#1103](https://github.com/cargo-generate/cargo-generate/pull/1103))
- Bump assert_cmd from 2.0.12 to 2.0.13 ([#1102](https://github.com/cargo-generate/cargo-generate/pull/1102))
- Bump bstr from 1.8.0 to 1.9.0 ([#1094](https://github.com/cargo-generate/cargo-generate/pull/1094))
- Switch to sccache ([#1126](https://github.com/cargo-generate/cargo-generate/pull/1126))
- Add Text and Editor type ([#1113](https://github.com/cargo-generate/cargo-generate/pull/1113))
- Add --skip-submodules flag to optionalize cloning git submodules ([#1112](https://github.com/cargo-generate/cargo-generate/pull/1112))
- Several versions ([#1130](https://github.com/cargo-generate/cargo-generate/pull/1130))
- Ensure github-actions are updated by dependabot ([#1134](https://github.com/cargo-generate/cargo-generate/pull/1134))

### 🤕 Fixes

- Fix multiple `Unreleased` versions in CHANGELOG.md
- Fix release pr should trigger builds
- Fix very strange old lint ([#1125](https://github.com/cargo-generate/cargo-generate/pull/1125))
- Release-plz config file breaking changes ([#1128](https://github.com/cargo-generate/cargo-generate/pull/1128))

## Unreleased

## [0.19.0] 2023-12-13

[0.19.0]: https://github.com/cargo-generate/cargo-generate/compare/0.18.5...0.19.0

### ✨ Features

- Add support for generating from a specific git revision of a git template ([#1070](https://github.com/cargo-generate/cargo-generate/pull/1070))
- Add release-plz pr ([#1076](https://github.com/cargo-generate/cargo-generate/pull/1076))
- Add cliff.toml config file to release-plz config

### 🛠️  Maintenance

- Bump fs-err from 2.9.0 to 2.11.0 ([#1063](https://github.com/cargo-generate/cargo-generate/pull/1063))
- Update Arch Linux package URL in installation.md ([#1064](https://github.com/cargo-generate/cargo-generate/pull/1064))
- Fine tune git-cliff config
- Fix git-cliff error on ci

### 🤕 Fixes

- Use a new token for creating the release and coverage data ([#1061](https://github.com/cargo-generate/cargo-generate/pull/1061))
- Convert destination to absolute path ([#1072](https://github.com/cargo-generate/cargo-generate/pull/1072))
- Fix gh-pages deploy token issue
- Fix empty define values ([#1078](https://github.com/cargo-generate/cargo-generate/pull/1078))
- Release-plz does not take the config file
- Hand release-plz config file over explicitly
- Extend the changelog config for git-cliff
- Reconfigure git-cliff a bit further
- Fix git-cliff changelog diff

## [0.18.5] 2023-11-11

[0.18.5]: https://github.com/cargo-generate/cargo-generate/compare/v0.18.4...v0.18.5

### 🤕 Fixes

- [fix(ci): fix clippy lint](https://github.com/cargo-generate/cargo-generate/pull/1052)
- [fix typo in readme](https://github.com/cargo-generate/cargo-generate/pull/1042)

### 🛠️ Maintenance

- [chore(deps): bump some deps](https://github.com/cargo-generate/cargo-generate/pull/1059)
- [chore(deps): bump git2 from 0.17.2 to 0.18.0](https://github.com/cargo-generate/cargo-generate/pull/1022)
- [chore(deps): bump clap from 4.3.24 to 4.4.2](https://github.com/cargo-generate/cargo-generate/pull/1021)

### ✨ Features

- [feat: support abbreviation for sourcehut](https://github.com/cargo-generate/cargo-generate/pull/1014)
- [Use auth-git2 for git authentication](https://github.com/cargo-generate/cargo-generate/pull/1025)
- [feat(error): use `fs_err` instead of `std::fs` to provide actionable error messages](https://github.com/cargo-generate/cargo-generate/pull/1055)

### Contributors

- [LukeMathWalker](https://github.com/LukeMathWalker)
- [de-vri-es](https://github.com/de-vri-es)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [n8henrie](https://github.com/n8henrie)
- [sassman](https://github.com/sassman)
- [sgeisenh](https://github.com/sgeisenh)


## [0.18.4] 2023-09-01

[0.18.4]: https://github.com/cargo-generate/cargo-generate/compare/v0.18.3...v0.18.4

### 🤕 Fixes

- [refactor(tests): improve tests (fixes #107)](https://github.com/cargo-generate/cargo-generate/pull/994)
- [fix: lints](https://github.com/cargo-generate/cargo-generate/pull/1016)
- [fix: lints](https://github.com/cargo-generate/cargo-generate/pull/985)
- [fix: use tempdir prefix](https://github.com/cargo-generate/cargo-generate/pull/978)
- [fix: update license field following SPDX 2.1 license expression standard](https://github.com/cargo-generate/cargo-generate/pull/984)
- [fix: -Z option build failures](https://github.com/cargo-generate/cargo-generate/pull/956)
- [fix: link to 'ignoring files'](https://github.com/cargo-generate/cargo-generate/pull/955)
- [fix: prompt order (#885)](https://github.com/cargo-generate/cargo-generate/pull/930)

### 🛠️ Maintenance

- [chore(deps): bump gix-config from 0.27.0 to 0.28.0 #1019](https://github.com/cargo-generate/cargo-generate/pull/1019)
- [chore(deps): bump several versions #1017](https://github.com/cargo-generate/cargo-generate/pull/1017)
- [chore(deps): bump url from 2.3.1 to 2.4.1](https://github.com/cargo-generate/cargo-generate/pull/1011)
- [chore(deps): bump tempfile from 3.7.0 to 3.8.0](https://github.com/cargo-generate/cargo-generate/pull/1010)
- [chore(deps): bump tempfile from ~3.5 to ~3.7](https://github.com/cargo-generate/cargo-generate/pull/992)
- [chore(deps): bump gix-config from 0.20.1 to 0.26.1](https://github.com/cargo-generate/cargo-generate/pull/991)
- [chore(deps): bump clap from 4.2.7 to 4.3.19](https://github.com/cargo-generate/cargo-generate/pull/990)
- [chore(deps): bump regex from 1.8.2 to 1.9.1](https://github.com/cargo-generate/cargo-generate/pull/982)
- [chore(deps): bump bstr from 1.4.0 to 1.6.0](https://github.com/cargo-generate/cargo-generate/pull/981)
- [chore(deps): bump rhai from 1.13.0 to 1.15.1](https://github.com/cargo-generate/cargo-generate/pull/972)
- [chore(deps): bump path-absolutize from 3.0.14 to 3.1.0](https://github.com/cargo-generate/cargo-generate/pull/938)

### ✨ Features

- [feat: show case how cargo-generate can be used as a library](https://github.com/cargo-generate/cargo-generate/pull/1015)

### Contributors

- [dependabot[bot]](https://github.com/apps/dependabot)
- [ealmloff](https://github.com/ealmloff)
- [frisoft](https://github.com/frisoft)
- [jonaro00](https://github.com/jonaro00)
- [sassman](https://github.com/sassman)
- [sshine](https://github.com/sshine)


## [0.18.3] 2023-05-24
[0.18.3]: https://github.com/cargo-generate/cargo-generate/compare/v0.18.2...v0.18.3

### 🤕 Fixes

- [Add type annotation to avoid ambiguity introduced in bstr 1.5.0 (#942)](https://github.com/cargo-generate/cargo-generate/pull/946)
- [fix: Move to the smaller, cargo-team maintained `home` crate](https://github.com/cargo-generate/cargo-generate/pull/925)

### 🛠️ Maintenance

- [chore(deps): bump openssl from 0.10.50 to 0.10.52](https://github.com/cargo-generate/cargo-generate/pull/929)
- [chore(deps): bump clap from 4.2.1 to 4.2.4](https://github.com/cargo-generate/cargo-generate/pull/928)
- [chore(deps): bump regex from 1.7.3 to 1.8.1](https://github.com/cargo-generate/cargo-generate/pull/927)
- [chore(deps): bump assert_cmd from 2.0.10 to 2.0.11](https://github.com/cargo-generate/cargo-generate/pull/924)
- [chore(deps): bump predicates from 3.0.2 to 3.0.3](https://github.com/cargo-generate/cargo-generate/pull/923)

### Contributors

- [Jacob640](https://github.com/Jacob640)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [utkarshgupta137](https://github.com/utkarshgupta137)


## [0.18.2] 2023-04-11
[0.18.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.18.1...v0.18.2

### 🤕 Fixes
- [fix: `.liquid` suffixes not stripped when cloning git templates](https://github.com/cargo-generate/cargo-generate/pull/916)
- [Migrate from println! to log+env_logger](https://github.com/cargo-generate/cargo-generate/pull/907)

### 🛠️ Maintenance
- [Update various dependencies](https://github.com/cargo-generate/cargo-generate/pull/919)

### Contributors
- [KaitlynEthylia](https://github.com/KaitlynEthylia)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [mothran](https://github.com/mothran)
- [sassman](https://github.com/sassman)

## [0.18.1] 2023-02-25
[0.18.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.18.0...v0.18.1

### 🛠️ Maintenance
- [chore(deps): bump remove_dir_all from 0.7.0 to 0.8.0](https://github.com/cargo-generate/cargo-generate/pull/886)

### 🤕 Fixes
- [make feature `vendored-openssl` opt in by default (#883)](https://github.com/cargo-generate/cargo-generate/pull/887)

### Contributors
- [dependabot[bot]](https://github.com/apps/dependabot)
- [sassman](https://github.com/sassman)

## [0.18.0] 2023-02-22
[0.18.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.6...v0.18.0

### ✨ Features
- [liquid filter for running a Rhai script (#879)](https://github.com/cargo-generate/cargo-generate/pull/866)
- [put vendored libraries behind features (#856)](https://github.com/cargo-generate/cargo-generate/pull/856)

### 🛠️ Maintenance
- [Update various dependencies](https://github.com/cargo-generate/cargo-generate/pull/878)

### 🤕 Fixes
- [update `git-config` to `gix-config` (#879)](https://github.com/cargo-generate/cargo-generate/pull/880)

### Contributors
- [Byron](https://github.com/Byron)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)
- [figsoda](https://github.com/figsoda)

## [0.17.6] 2023-01-19
[0.17.6]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.5...v0.17.6

### 🛠️ Maintenance
- [chore(deps): bump git2 from 0.15.0 to 0.16.0](https://github.com/cargo-generate/cargo-generate/pull/851)

### 🤕 Fixes
- [fix(ci): fix deb package on release (#852)](https://github.com/cargo-generate/cargo-generate/pull/853)
- release notes for 0.17.5 missed out on #844

### Contributors
- [dependabot[bot]](https://github.com/apps/dependabot)
- [sassman](https://github.com/sassman)

## [0.17.5] 2023-01-16
[0.17.5]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.4...v0.17.5

### 🛠️ Maintenance
- [chore(deps): bump dependencies](https://github.com/cargo-generate/cargo-generate/pull/850)

### ✨ Features
- [build a debian package on release](https://github.com/cargo-generate/cargo-generate/pull/846)
- [show a warning to the user if the template doesn't agree with the cli parameters](https://github.com/cargo-generate/cargo-generate/pull/844)

### Contributors
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)

## [0.17.4] 2022-12-14
[0.17.4]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.3...v0.17.4

### 🤕 Fixes
- [fix(git:url): fix #752 short `owner/repo` github urls](https://github.com/cargo-generate/cargo-generate/pull/821)
- [fix(docs): fix outdated docs, relates to #674](https://github.com/cargo-generate/cargo-generate/pull/818)

### 🛠️ Maintenance
- [chore(deps): bump assert_cmd from 2.0.6 to 2.0.7](https://github.com/cargo-generate/cargo-generate/pull/815)
- [chore(deps): bump predicates from 2.1.2 to 2.1.4](https://github.com/cargo-generate/cargo-generate/pull/814)
- [chore(deps): bump clap from 4.0.22 to 4.0.29](https://github.com/cargo-generate/cargo-generate/pull/813)
- [chore(deps): bump serde from 1.0.147 to 1.0.149](https://github.com/cargo-generate/cargo-generate/pull/812)
- [chore(deps): bump git-config from 0.11.0 to 0.12.0](https://github.com/cargo-generate/cargo-generate/pull/810)
- [chore(deps): bump openssl from 0.10.42 to 0.10.43](https://github.com/cargo-generate/cargo-generate/pull/809)
- [chore(deps): bump git-config from 0.10.0 to 0.11.0](https://github.com/cargo-generate/cargo-generate/pull/804)

### Contributors
- [sassman](https://github.com/sassman)

## [0.17.3] 2022-11-11
[0.17.3]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.2...v0.17.3

### 🤕 Fixes
- [fix(#795): '--list-favorites' option is broken (#796) ](https://github.com/cargo-generate/cargo-generate/pull/796)

### Contributors
- [taurr](https://github.com/taurr)
 
## [0.17.2] 2022-11-09
[0.17.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.1...v0.17.2

### 🤕 Fixes
- [fix(#793): empty defaults are now back #794 ](https://github.com/cargo-generate/cargo-generate/pull/794)

### Contributors
- [sassman](https://github.com/sassman)
 
## [0.17.1] 2022-11-07
[0.17.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.17.0...v0.17.1

### 🤕 Fixes
- [fix(#790): fix string template var prompts and default value presentation issues](https://github.com/cargo-generate/cargo-generate/pull/791)

### 🛠️ Maintenance
- [chore(deps): bump clap from 4.0.19 to 4.0.22](https://github.com/cargo-generate/cargo-generate/pull/792)
 
### Contributors
- [sassman](https://github.com/sassman)

## [0.17.0] 2022-11-07
[0.17.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.16.0...v0.17.0

### ✨ Features
- [Add `aarch64-apple-darwin` release package](https://github.com/cargo-generate/cargo-generate/pull/775)
- [Extend the GitHub CI build pipeline for spellchecks](https://github.com/cargo-generate/cargo-generate/pull/764)
- [ci: close issues that are labeled as `waiting-for-user-input` and got no updates for 14days](https://github.com/cargo-generate/cargo-generate/pull/762)
- [feat: impl `Default` for `GenerateArgs` and `TemplatePath`](https://github.com/cargo-generate/cargo-generate/pull/754)
- [support providing `project-name` via hooks, env vars and template value file](https://github.com/cargo-generate/cargo-generate/pull/729)
- [Remove `Smoke Test` step](https://github.com/cargo-generate/cargo-generate/pull/721)
- [renaming files by using e.g. `{{project-name}}.yml` as filename doesn't remove the original file](https://github.com/cargo-generate/cargo-generate/pull/713)
- [Support `--test` for running tests on the expanded template](https://github.com/cargo-generate/cargo-generate/pull/699)
 
### 🤕 Fixes
- [Variables set in pre scripts don't carry over to the template](https://github.com/cargo-generate/cargo-generate/pull/711)
- [Git default branch is not honored when using --path](https://github.com/cargo-generate/cargo-generate/pull/712)
- [Deleting_a_non_existing_file_should_not_fail](https://github.com/cargo-generate/cargo-generate/pull/730)
- [Fix typos](https://github.com/cargo-generate/cargo-generate/pull/751)
- [Fix link to Github Action](https://github.com/cargo-generate/cargo-generate/pull/742)
- [Fix tests (1.65.0 toolchain support + git 2.38.1 compatibility)](https://github.com/cargo-generate/cargo-generate/pull/780)
- [When using --init, do not copy .git directories](https://github.com/cargo-generate/cargo-generate/pull/781)

### 🛠️ Maintenance
- [chore(deps): bump git-config from 0.5.0 to 0.10.0](https://github.com/cargo-generate/cargo-generate/pull/786)
- [chore(deps): bump clap from 3.2.22 to 4.0](https://github.com/cargo-generate/cargo-generate/pull/785)
- [Dependency_update](https://github.com/cargo-generate/cargo-generate/pull/728)
- [chore(deps): bump semver from 1.0.12 to 1.0.13](https://github.com/cargo-generate/cargo-generate/pull/727)
- [chore(deps): bump indoc from 1.0.6 to 1.0.7](https://github.com/cargo-generate/cargo-generate/pull/725)
- [chore(deps): bump paste from 1.0.7 to 1.0.8](https://github.com/cargo-generate/cargo-generate/pull/724)
- [chore(deps): bump thiserror from 1.0.31 to 1.0.32](https://github.com/cargo-generate/cargo-generate/pull/723)
- [chore(deps): bump serde from 1.0.141 to 1.0.142](https://github.com/cargo-generate/cargo-generate/pull/722)
- [chore(deps): bump serde from 1.0.140 to 1.0.141](https://github.com/cargo-generate/cargo-generate/pull/720)
- [chore(deps): bump git2 from 0.14.4 to 0.15.0](https://github.com/cargo-generate/cargo-generate/pull/719)
- [chore(deps): bump dialoguer from 0.10.1 to 0.10.2](https://github.com/cargo-generate/cargo-generate/pull/718)
- [chore(deps): bump clap from 3.2.15 to 3.2.16](https://github.com/cargo-generate/cargo-generate/pull/717)
- [chore(deps): bump anyhow from 1.0.58 to 1.0.59](https://github.com/cargo-generate/cargo-generate/pull/715)
- [chore(deps): bump console from 0.15.0 to 0.15.1](https://github.com/cargo-generate/cargo-generate/pull/714)
- [chore(deps): bump clap from 3.2.14 to 3.2.15](https://github.com/cargo-generate/cargo-generate/pull/705)
- [chore(deps): bump clap from 3.2.8 to 3.2.12](https://github.com/cargo-generate/cargo-generate/pull/702)
- [update dependencies](https://github.com/cargo-generate/cargo-generate/pull/755)
- [Removed Serenity Template from Template List](https://github.com/cargo-generate/cargo-generate/pull/707)

### Contributors
- [EstebanBorai](https://github.com/EstebanBorai)
- [MalloryA](https://github.com/MalloryA)
- [SergioGasquez](https://github.com/SergioGasquez)
- [chilipepperhott](https://github.com/chilipepperhott)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [joshrotenberg](https://github.com/joshrotenberg)
- [kianmeng](https://github.com/kianmeng)
- [printfn](https://github.com/printfn)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)
- [turboMaCk](https://github.com/turboMaCk)

## [0.16.0] 2022-07-25
[0.16.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.15.2...v0.16.0

### ✨ Features
- [Support `--test` for running tests on the expanded template](https://github.com/cargo-generate/cargo-generate/pull/699)
- [Allow the template author to manually specify sub-temlates](https://github.com/cargo-generate/cargo-generate/pull/693)

  Sub-templates may now be specified in the `cargo-generate.toml` file like this:
  ```toml
  [template]
  sub_templates = ["sub1", "sub2"]
  ```
  Doing so also sets the order when `cargo-generate` asks what to expand, while the first option will be the default.
  
  If a selected template doesn't have a `cargo-generate.toml` file, but a parent one exists, any configured sub-templates will be ignored.
  
  Further implication is that sub-templates no longer needs to have a `cargo-generate.toml` file.
- [Test for file existance from rhai scripts](https://github.com/cargo-generate/cargo-generate/pull/690)
  Adds the `file::exists(path: &str)` method for use from hook scripts.
- Conditionals and placeholders are now supported at multiple levels.
  If a template sets up more placeholders conditionally, those placeholders are now checked/asked
  for and respected for use in further expressions/conditionals.
- [Set default VCS in the favorite or template config](https://github.com/cargo-generate/cargo-generate/issues/635)
- [Add case filters to `liquid` and functions to `Rhai`. Enables case changing functionality from both `liquid` and `Rhai`](https://github.com/cargo-generate/cargo-generate/issues/638)
- [Add placeholder `{{is_init}}`. Enables templates to adjust if they are expanded with the `--init` arg for `cargo-generate`](https://github.com/cargo-generate/cargo-generate/issues/649)
- [Make it possible to convert to `snake_case` and `kebab_case` from `Rhai`](https://github.com/cargo-generate/cargo-generate/pull/662)
- [New `Rhai` variable: `is_init` ?](https://github.com/cargo-generate/cargo-generate/pull/661)
- [Return the path of the generated project from the `generate` function](https://github.com/cargo-generate/cargo-generate/pull/666)
- [Support `--tag` in addition to `--branch`](https://github.com/cargo-generate/cargo-generate/pull/671)
- [Set default VCS from within templates](https://github.com/cargo-generate/cargo-generate/pull/677)
- [Let template specify it is designed to NOT go into a sub-folder (aka. the template always has --init behaviour)](https://github.com/cargo-generate/cargo-generate/pull/680)
- [Add an option to overwrite files in an existing project](https://github.com/cargo-generate/cargo-generate/pull/691)
- [Add a link to finding templates directly from the README](https://github.com/cargo-generate/cargo-generate/pull/700)

### 🤕 Fixes
- [fix(#514): boolean value being ignored](https://github.com/cargo-generate/cargo-generate/pull/669)
- [fix lint error](https://github.com/cargo-generate/cargo-generate/pull/678)
- [fix for cargo-generate changes the $CWD](https://github.com/cargo-generate/cargo-generate/pull/688)
- [should fail if --define placeholder value doesn't match placeholder regex](https://github.com/cargo-generate/cargo-generate/pull/692)

### 🛠️ Maintenance
- [chore(deps): bump anyhow from 1.0.57 to 1.0.58](https://github.com/cargo-generate/cargo-generate/pull/663)
- [chore(deps): bump clap from 3.2.5 to 3.2.6](https://github.com/cargo-generate/cargo-generate/pull/672)
- [chore(deps): bump clap from 3.2.6 to 3.2.8](https://github.com/cargo-generate/cargo-generate/pull/684)
- [chore(deps): bump clap from 3.2.8 to 3.2.12](https://github.com/cargo-generate/cargo-generate/pull/702)
- [chore(deps): bump openssl-src from 111.20.0+1.1.1o to 111.22.0+1.1.1q](https://github.com/cargo-generate/cargo-generate/pull/687)
- [chore(deps): bump rhai from 1.7.0 to 1.8.0](https://github.com/cargo-generate/cargo-generate/pull/683)
- [chore(deps): bump serde from 1.0.137 to 1.0.138](https://github.com/cargo-generate/cargo-generate/pull/682)
- [chore(deps): bump semver from 1.0.10 to 1.0.12](https://github.com/cargo-generate/cargo-generate/pull/681)
- [chore(deps): bump serde from 1.0.138 to 1.0.139](https://github.com/cargo-generate/cargo-generate/pull/697)
- [chore(deps): bump openssl from 0.10.40 to 0.10.41](https://github.com/cargo-generate/cargo-generate/pull/696)
- [chore(deps): bump regex from 1.5.6 to 1.6.0](https://github.com/cargo-generate/cargo-generate/pull/695)
- [refactor: use impl T to improve readability, and change path params to use Path](https://github.com/cargo-generate/cargo-generate/pull/694)

### Contributors
- [taurr](https://github.com/taurr)
- [sassman](https://github.com/sassman)
- [yozhgoor](https://github.com/yozhgoor)
- [dependabot[bot]](https://github.com/apps/dependabot)

## [0.15.2] 2022-06-16
[0.15.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.15.1...v0.15.2

### 🤕 Fixes
- [fix(#657): fix smoke tests + add smoke test to the build pipeline](https://github.com/cargo-generate/cargo-generate/pull/658)

### Contributors
- [sassman](https://github.com/sassman)


## [0.15.1] 2022-06-16
[0.15.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.15.0...v0.15.1

### 🤕 Fixes
- [Add `ssh-agent` support and fix windows git+ssh issues](https://github.com/cargo-generate/cargo-generate/pull/655)
- [`cargo-generate` without options panics and fix `--help`](https://github.com/cargo-generate/cargo-generate/pull/651)

### Contributors
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)


## [0.15.0] 2022-06-15
[0.15.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.14.0...v0.15.0

### ✨ Features
- [Add placeholder `{{within_cargo_project}}`. Enables templates to adust if they are expanded inside a `cargo` project](https://github.com/cargo-generate/cargo-generate/pull/628)
- [output from a *POST* `rhai` script is seemingly output *before* expansion.](https://github.com/cargo-generate/cargo-generate/pull/640)
- [auto remove `.liquid` file extensions if present](https://github.com/cargo-generate/cargo-generate/pull/639)
- [Generate release packages for `aarch64-unknown-linux-gnu`](https://github.com/cargo-generate/cargo-generate/pull/620)
- [Enhance canonicalize_path and git errors](https://github.com/cargo-generate/cargo-generate/pull/595)

### 🛠️ Maintenance
- [chore(deps): upgrade to latest clap version + fix deprecations](https://github.com/cargo-generate/cargo-generate/pull/645)
- [chore(deps): bump git-config from 0.4.0 to 0.5.0](https://github.com/cargo-generate/cargo-generate/pull/641)
- [chore(deps): bump git-config from 0.2.0 to 0.4.0](https://github.com/cargo-generate/cargo-generate/pull/626)
- [chore(deps): bump sanitize-filename from 0.3.0 to 0.4.0](https://github.com/cargo-generate/cargo-generate/pull/591)
- [Override project_dir when using `generate()`](https://github.com/cargo-generate/cargo-generate/pull/625)
- [Replace `structopt` by `clap`](https://github.com/cargo-generate/cargo-generate/pull/624)
- [fix(docs): related to `v0.14.0` some breaking required doc changes](https://github.com/cargo-generate/cargo-generate/pull/622)
- [Avoid panic caused by deleting git folders](https://github.com/cargo-generate/cargo-generate/pull/621)

### Contributors
- [SergioGasquez](https://github.com/SergioGasquez)
- [andrewjstone](https://github.com/andrewjstone)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [jm-observer](https://github.com/jm-observer)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)
- [yozhgoor](https://github.com/yozhgoor)


## [0.14.0] 2022-05-31
[0.14.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.13.1...v0.14.0

### ✨ Features
- [Suppress misleading git initialization message](https://github.com/cargo-generate/cargo-generate/pull/606)
- [Generate release packages for `x86_64-unknown-linux-gnu`](https://github.com/cargo-generate/cargo-generate/pull/602)
- [Allow system commands](https://github.com/cargo-generate/cargo-generate/pull/598)
- [Less noise](https://github.com/cargo-generate/cargo-generate/pull/597)
- [Enhance canonicalize_path and git errors](https://github.com/cargo-generate/cargo-generate/pull/595)
- [Refactor of handling favorites, git and path](https://github.com/cargo-generate/cargo-generate/pull/572)

### 🛠️ Maintenance
- [chore(deps): bump sanitize-filename from 0.3.0 to 0.4.0](https://github.com/cargo-generate/cargo-generate/pull/591)
- [chore(deps): bump anyhow from 1.0.56 to 1.0.57](https://github.com/cargo-generate/cargo-generate/pull/587)
- [chore(deps): bump toml from 0.5.8 to 0.5.9](https://github.com/cargo-generate/cargo-generate/pull/585)

### Contributors
- [SergioGasquez](https://github.com/SergioGasquez)
- [andrewjstone](https://github.com/andrewjstone)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [remlse](https://github.com/remlse)
- [sassman](https://github.com/sassman)
- [v-n-suadicani-issuu](https://github.com/v-n-suadicani-issuu)


## [0.13.1] 2022-04-11 
[0.13.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.13.0...v0.13.1

### 🛠️ Maintenance
- [chore(deps): bump git-config from 0.1.11 to 0.2.1](https://github.com/cargo-generate/cargo-generate/pull/581)
- [chore(deps): bump liquid* from 0.24 to 0.26](https://github.com/cargo-generate/cargo-generate/pull/579)
- [chore(deps): bump rhai from 1.5.0 to 1.6.0](https://github.com/cargo-generate/cargo-generate/pull/570)
- [chore(deps): bump versions](https://github.com/cargo-generate/cargo-generate/pull/566)
- [chore(deps): bump path-absolutize from 3.0.11 to 3.0.12](https://github.com/cargo-generate/cargo-generate/pull/565)
- [chore(deps): bump git2 version to 0.14](https://github.com/cargo-generate/cargo-generate/pull/563)
- [chore(deps): bump regex from 1.5.4 to 1.5.5](https://github.com/cargo-generate/cargo-generate/pull/561)
- [chore(deps): bump liquid-derive from 0.23.1 to 0.24.0](https://github.com/cargo-generate/cargo-generate/pull/560)
- [chore(deps): bump anyhow from 1.0.53 to 1.0.56](https://github.com/cargo-generate/cargo-generate/pull/556)
- [chore(deps): bump semver from 1.0.5 to 1.0.6](https://github.com/cargo-generate/cargo-generate/pull/552)
- [chore(deps): bump rhai from 1.4.1 to 1.5.0](https://github.com/cargo-generate/cargo-generate/pull/551)
- [chore(deps): bump dialoguer from 0.9.0 to 0.10.0](https://github.com/cargo-generate/cargo-generate/pull/548)
- [chore(clippy): make clippy happy](https://github.com/cargo-generate/cargo-generate/pull/564)
- [chore(docs): added git url to book.toml](https://github.com/cargo-generate/cargo-generate/pull/562)

### 🤕 Fixes
- [`--config` should behave relative to `CWD`](https://github.com/cargo-generate/cargo-generate/pull/544)

### Contributors
- [TonalidadeHidrica](https://github.com/TonalidadeHidrica)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [saikiran1043](https://github.com/saikiran1043)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)
- [xoac](https://github.com/xoac)

## [0.13.0] 2022-02-08
[0.13.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.12.0...v0.13.0

### ✨ Features
- [feat(#299): cargo generate book](https://github.com/cargo-generate/cargo-generate/pull/519)
- [feat(chat): add matrix chat and logo draft](https://github.com/cargo-generate/cargo-generate/pull/537)
- [feat(#418) Make `--identity` configurable in cargo-generate.toml and verbose error message](https://github.com/cargo-generate/cargo-generate/pull/534)
- [feat(#520): Support Git's https to ssh rewriting configuration](https://github.com/cargo-generate/cargo-generate/pull/533)
- [feat(#526): template sub directory error lacks context](https://github.com/cargo-generate/cargo-generate/pull/529)
- [feat(#516): new `--force-git-init` flag](https://github.com/cargo-generate/cargo-generate/pull/525)

### 🤕 Fixes
- [fix(abbrev): fix index out of range for git abbrev](https://github.com/cargo-generate/cargo-generate/pull/524)
- [fix(#79): improve the error message in case the repo or user does not exists](https://github.com/cargo-generate/cargo-generate/pull/521)
- [fix(#510): Using `--path` bring the `.git` folder from the source with it](https://github.com/cargo-generate/cargo-generate/pull/518)

### 🛠️ Maintenance
- [chore(book): minor fixes of broken links](https://github.com/cargo-generate/cargo-generate/pull/538)
- [chore(deps): bump semver from 1.0.4 to 1.0.5](https://github.com/cargo-generate/cargo-generate/pull/536)
- [chore(deps): bump tempfile from 3.2.0 to 3.3.0](https://github.com/cargo-generate/cargo-generate/pull/515)
- [ci: remove reference to unused action](https://github.com/cargo-generate/cargo-generate/pull/503)

## Contributors

- [MarcoIeni](https://github.com/MarcoIeni)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [joshrotenberg](https://github.com/joshrotenberg)
- [sassman](https://github.com/sassman)


## [0.12.0] 2022-02-04
[0.12.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.11.1...v0.12.0

### 🛠️ Maintenance
- [upgrade deps + migrate to the new api of hecks](https://github.com/cargo-generate/cargo-generate/pull/512)
- [migrate default branch from master to main](https://github.com/cargo-generate/cargo-generate/pull/513)

### ✨ Features
- [abbreviation support for gitlab and bitbucket users](https://github.com/cargo-generate/cargo-generate/pull/511), fixes #391

### Contributors
- [sassman](https://github.com/sassman)

## [0.11.1] 2021-12-03
[0.11.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.11.0...v0.11.1

### 🛠️ Maintenance
- [update dependencies + upgrade to rust 2021 edition](https://github.com/cargo-generate/cargo-generate/pull/500)

### Contributors
- [sassman](https://github.com/sassman)

## [0.11.0] 2021-11-07
[0.11.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.10.3...v0.11.0

### ✨ Features
- [feature: add support for arrays in the variable::set Rhai script extension](https://github.com/cargo-generate/cargo-generate/pull/474)
- [feature: add username placeholder](https://github.com/cargo-generate/cargo-generate/pull/463)

### 🤕 Fixes
- [fix: print regex error](https://github.com/cargo-generate/cargo-generate/pull/488)
- [fix: README regex example](https://github.com/cargo-generate/cargo-generate/pull/490)
- [fix: improved choice lists](https://github.com/cargo-generate/cargo-generate/issues/400)

### 🛠️ Maintenance
- [update dependencies](https://github.com/cargo-generate/cargo-generate/pull/491)
- [whitelist `time` advisories](https://github.com/cargo-generate/cargo-generate/pull/489)

### Contributors
- [GabrielDertoni](https://github.com/GabrielDertoni)
- [MarcoIeni](https://github.com/MarcoIeni)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)

## [0.10.3] 2021-10-11
[0.10.3]: https://github.com/cargo-generate/cargo-generate/compare/v0.10.2...v0.10.3

### 🛠️ Maintenance
- [update dependencies](https://github.com/cargo-generate/cargo-generate/pull/480)

## [0.10.2] 2021-10-09

[0.10.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.10.1...v0.10.2

### 🤕 Fixes
- [fix(git+libgit2:versions): try to fix deps issue](https://github.com/cargo-generate/cargo-generate/pull/477)
- [fix: Rhai variable extension not updating Liquid Object](https://github.com/cargo-generate/cargo-generate/pull/472)
- [fix: `cargo-generate.toml` not picked up on interactive template selection](https://github.com/cargo-generate/cargo-generate/pull/469)

### ✨ Features
- [improvement: use a choice prompt instead of strings for boolean placeholders](https://github.com/cargo-generate/cargo-generate/pull/467)

### Contributors
- [GabrielDertoni](https://github.com/GabrielDertoni)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)

## [0.10.1] 2021-09-23

[0.10.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.10.0...v0.10.1

### 🤕 Fixes
- [improve ergonomics when asking for project-name](https://github.com/cargo-generate/cargo-generate/pull/464)
  by [@taurr](https://github.com/taurr)
- [revert_to_git2_0.13.21_to_avoid_issues](https://github.com/cargo-generate/cargo-generate/pull/462)
  by [@taurr](https://github.com/taurr)

## [0.10.0] 2021-09-19
### ✨ Features
- [feat: prompt for subfolder upon multiple templates](https://github.com/cargo-generate/cargo-generate/pull/446)
  ![demo](https://user-images.githubusercontent.com/7338549/132187949-befdcaf3-a9ba-426b-b530-e4d6046a8c3d.gif)

  by [@taurr](https://github.com/taurr)
- [feat: Pre/Post hooks](https://github.com/cargo-generate/cargo-generate/pull/445), 
  [issue/18](https://github.com/cargo-generate/cargo-generate/issues/18)

  Support for template hooks written in [rhai](https://rhai.rs/book/about/index.html).

  Enables the template author to e.g. create/modify/prompt for template variables using complex logic, or to create/delete/rename files within the template.

  by [@taurr](https://github.com/taurr)
- [feat: include files conditionally](https://github.com/cargo-generate/cargo-generate/pull/431)
  by [@taurr](https://github.com/taurr)

### 🛠️ Maintenance
- [fix(docs): closes #447](https://github.com/cargo-generate/cargo-generate/pull/457)
- [Save 33% in size on release binary, bump dependencies](https://github.com/cargo-generate/cargo-generate/pull/454)
- [Remove cargo dependency](https://github.com/cargo-generate/cargo-generate/pull/444)
- [chore(deps): bump thiserror from 1.0.26 to 1.0.28](https://github.com/cargo-generate/cargo-generate/pull/438)
- [refactor: add more clippy warnings](https://github.com/cargo-generate/cargo-generate/pull/437)
- [README: improve cargo_generate_version example](https://github.com/cargo-generate/cargo-generate/pull/428)
- [README: specify minimum version for feature](https://github.com/cargo-generate/cargo-generate/pull/433)
- [README: Added --path documentation](https://github.com/cargo-generate/cargo-generate/pull/435)
- [README: add installation instructions for Arch Linux](https://github.com/cargo-generate/cargo-generate/pull/436)
- [README: add a little starter guide for template ci testing](https://github.com/cargo-generate/cargo-generate/pull/458)
- 
### Contributors
- [MarcoIeni](https://github.com/MarcoIeni)
- [dependabot[bot]](https://github.com/apps/dependabot)
- [k0pernicus](https://github.com/k0pernicus)
- [messense](https://github.com/messense)
- [orhun](https://github.com/orhun)
- [sassman](https://github.com/sassman)
- [taurr](https://github.com/taurr)
  
## [0.9.0] 2021-08-24
### ✨ Features
- [enable paging for long choice lists](https://github.com/cargo-generate/cargo-generate/pull/423)
  
  improving the handling of very long choice lists long
  [see #400](https://github.com/cargo-generate/cargo-generate/issues/400), by [@taurr](https://github.com/taurr)

- [template parsing: handle undefined variables gracefully](https://github.com/cargo-generate/cargo-generate/pull/422)

  Undefined template variables do not cause any breaking of the template generation any longer. They will be kept as they are.

  [see #204](https://github.com/cargo-generate/cargo-generate/issues/204),
  [#205](https://github.com/cargo-generate/cargo-generate/issues/205),
  by [@sassman](https://github.com/sassman)

- [TEMPLATES.md: link to cargo-generate topic](https://github.com/cargo-generate/cargo-generate/pull/416)

  Template repos should be tagged with the `cargo-generate` GitHub topic, [read more..](https://github.com/cargo-generate/cargo-generate/blob/main/TEMPLATES.md#available-templates)

  by [@MarcoIeni](https://github.com/MarcoIeni),
  [#407](https://github.com/cargo-generate/cargo-generate/pull/407)
  by [@sassman](https://github.com/sassman)

- [Add `init` like behavior.](https://github.com/cargo-generate/cargo-generate/pull/414)

  a template can now be generated into the current dir, without a git init or anything, [read more..](https://github.com/cargo-generate/cargo-generate#generating-into-current-dir)

  [see #193](https://github.com/cargo-generate/cargo-generate/issues/193),
  by [@taurr](https://github.com/taurr)

- [Allow version requirement in `cargo-generate.toml`](https://github.com/cargo-generate/cargo-generate/pull/413)

  a template can now define the compatible cargo generate version number requirement as a requirement, [read more..](https://github.com/cargo-generate/cargo-generate#require-cargo-generate-version-from-template)
  [see #76](https://github.com/cargo-generate/cargo-generate/issues/76),
  by [@taurr](https://github.com/taurr)

- [Allow cargo-generate.toml as alternative to cargo-generate.](https://github.com/cargo-generate/cargo-generate/pull/412), 
  by [@taurr](https://github.com/taurr)

- [fix(tests:linux): use the current directory for canonicalize result](https://github.com/cargo-generate/cargo-generate/pull/411), 
  by [@orhun](https://github.com/orhun)

- [Introduce `--path` flag](https://github.com/cargo-generate/cargo-generate/pull/410)

  supporting now local folders (that are not under git) as templates via `--path <local-folder>`

  [see #406](https://github.com/cargo-generate/cargo-generate/issues/406),
  [#47](https://github.com/cargo-generate/cargo-generate/issues/47)

  also [#390](https://github.com/cargo-generate/cargo-generate/pull/390),
  [#387](https://github.com/cargo-generate/cargo-generate/issues/387)
  by [@taurr](https://github.com/taurr)

- [Allow specification of default template values](https://github.com/cargo-generate/cargo-generate/pull/409)

  default values for template variables can now be defined on several levels, external file, in the favorites configuration and via environment variables, [read more..](https://github.com/cargo-generate/cargo-generate#default-values-for-placeholders-from-a-file)
  
  [see #389](https://github.com/cargo-generate/cargo-generate/issues/389),
  [#46](https://github.com/cargo-generate/cargo-generate/issues/46),
  by [@taurr](https://github.com/taurr)

- [template variables via environment variables](https://github.com/cargo-generate/cargo-generate/pull/404)

  [#389](https://github.com/cargo-generate/cargo-generate/issues/389),
  by [@taurr](https://github.com/taurr)

- [template variables in Directories](https://github.com/cargo-generate/cargo-generate/pull/397), 
  [#396](https://github.com/cargo-generate/cargo-generate/issues/396)

  supporting now template variables as directory names like `{{project_name}}/{{project_name}}.rs`

  by [dave-tucker](https://github.com/dave-tucker)
  and [#395](https://github.com/cargo-generate/cargo-generate/pull/395)
  by [@taurr](https://github.com/taurr)

- [Allow `--define` to specify values](https://github.com/cargo-generate/cargo-generate/pull/392)
  [#389](https://github.com/cargo-generate/cargo-generate/issues/389)
  
  command line argument `--define` to define template variables
  
  by [@taurr](https://github.com/taurr)

- [Improve output of --list-favorites](https://github.com/cargo-generate/cargo-generate/pull/388)
  by [@taurr](https://github.com/taurr)

### 🛠️ Maintenance
- [chore(deps): bump openssl from 0.10.35 to 0.10.36](https://github.com/cargo-generate/cargo-generate/pull/429)
- [README: improve cargo_generate_version example](https://github.com/cargo-generate/cargo-generate/pull/428)
- [ci: add documentation check](https://github.com/cargo-generate/cargo-generate/pull/426)
- [fix additional clippy lints](https://github.com/cargo-generate/cargo-generate/pull/425)
- [Fix typo in README.md](https://github.com/cargo-generate/cargo-generate/pull/399)
- [Only use cross for linux prebuilt binaries](https://github.com/cargo-generate/cargo-generate/pull/398)
- [chore(deps): bump anyhow from 1.0.42 to 1.0.43](https://github.com/cargo-generate/cargo-generate/pull/421)
- [chore(deps): bump predicates from 2.0.1 to 2.0.2](https://github.com/cargo-generate/cargo-generate/pull/420)
- [chore(deps): bump git2 from 0.13.20 to 0.13.21](https://github.com/cargo-generate/cargo-generate/pull/419)
- [chore(deps): bump cargo from 0.54.0 to 0.55.0](https://github.com/cargo-generate/cargo-generate/pull/394)
- [chore(deps): bump assert_cmd from 1.0.7 to 1.0.8](https://github.com/cargo-generate/cargo-generate/pull/393)
- [chore(deps): bump assert_cmd from 1.0.8 to 2.0.0](https://github.com/cargo-generate/cargo-generate/pull/403)

## [0.8.0] 2021-07-30

- ### ✨ Features
  - [feat(template:filenames): template filenames now](https://github.com/cargo-generate/cargo-generate/pull/386)

    solves [#159](https://github.com/cargo-generate/cargo-generate/issues/159)

    A template author can use those placeholders in file names.  
    For example, a project named `awesome`, with a file name `{{project_name}}.rs` will be transformed to `awesome.rs`
    during generation. [read more..](https://github.com/cargo-generate/cargo-generate#templates)

    by [@sassman](https://github.com/sassman)

  - [feat(template:choices): better ux for template choice parameters](https://github.com/cargo-generate/cargo-generate/pull/381)

    for templates that contains parameters with a list of options + default option, users can now choose items from the
    list via the arrow keys and don't need manual typing anymore

    ![showcase](https://user-images.githubusercontent.com/329682/126956639-90d3c733-3813-4117-b305-89a308731c0b.png)

    by [@sassman](https://github.com/sassman)

  - [Feature: specify subdir in repo as template](https://github.com/cargo-generate/cargo-generate/pull/379)

    solves / relates to [#47](https://github.com/cargo-generate/cargo-generate/issues/47)
    [#78](https://github.com/cargo-generate/cargo-generate/issues/78)
    [#211](https://github.com/cargo-generate/cargo-generate/issues/211)
    [#291](https://github.com/cargo-generate/cargo-generate/issues/291)

    #### TL;DR: a subfolder (e.g. `examples/template`) of a given git repo can be used for templating

    In the following example we assume the subfolder `tests/integration/helpers` contain some template:
      ```sh
      cargo generate --git https://github.com/cargo-generate/cargo-generate   tests/integration/helpers
      ```

    The great thing is that library / tool authors are now enabled to ship their templates as part of e.g.
    the `examples/` folder directly from within their main repository.

    by [@taurr](https://github.com/taurr)

- ### 🛠️ Maintenance
  - [chore(deps): bump predicates from 2.0.0 to 2.0.1](https://github.com/cargo-generate/cargo-generate/pull/385)
  - [fix(ci): temp. workaround for GH Actions limitations](https://github.com/cargo-generate/cargo-generate/pull/382)
  - [Add testcase for #201 (Test git history is removed)](https://github.com/cargo-generate/cargo-generate/pull/380)
    by [taurr](https://github.com/taurr)

## [0.7.2] 2021-07-22

- ### ✨ Features
  - [Prebuilt binaries workfow for common targets](https://github.com/cargo-generate/cargo-generate/pull/377)
    by [@jashandeep-sohi](https://github.com/jashandeep-sohi)
    Whenever a GitHub Release is published, cargo-generate binaries are build and attached to the release.

## [0.7.1] 2021-07-18

- ### 🤕 Fixes
  - [Error on Windows: Git Error: Error cleaning up cloned template #375](https://github.com/cargo-generate/cargo-generate/issues/375)
    ,
    [pull/376](https://github.com/cargo-generate/cargo-generate/pull/376), by [@sassman](https://github.com/sassman)
  - [fix: fix InvalidPlaceholderName error string](https://github.com/cargo-generate/cargo-generate/pull/374)
    by [@NOBLES5E](https://github.com/NOBLES5E)

## [0.7.0] 2021-07-13

- ### ✨ Features
  - [feat(remote:ssh): support for ssh remote urls](https://github.com/cargo-generate/cargo-generate/pull/372)

Finally, `cargo-generate` supports git ssh remote
urls. [Read more in the docs](https://github.com/cargo-generate/cargo-generate#git-over-ssh)

- [feat(http:proxy): support http_proxy env vars](https://github.com/cargo-generate/cargo-generate/pull/342)

The typically known environment variables `HTTP_PROXY` and `HTTPS_PROXY` are now supported by `cargo-generate`.
[Read more in the docs](https://github.com/cargo-generate/cargo-generate#https-proxy)

- [feat(progress-bar): more useful progress bar](https://github.com/cargo-generate/cargo-generate/pull/339)
- [feat(crate-types): crate types --lib and --bin as in cargo init](https://github.com/cargo-generate/cargo-generate/pull/326)

Similar to `cargo init --lib`, a `crate-type` support is now there.
[Read more in the docs](https://github.com/cargo-generate/cargo-generate#example-for---bin-and---lib)

- [Add serenity template](https://github.com/cargo-generate/cargo-generate/pull/324)
- [Upgrade to GitHub-native Dependabot](https://github.com/cargo-generate/cargo-generate/pull/331)

- ### 🛠️ Maintenance
  - [chore(deps): bump anyhow from 1.0.41 to 1.0.42](https://github.com/cargo-generate/cargo-generate/pull/371)
  - [chore(deps): bump predicates from 1.0.8 to 2.0.0](https://github.com/cargo-generate/cargo-generate/pull/366)
  - [chore(deps): bump thiserror from 1.0.25 to 1.0.26](https://github.com/cargo-generate/cargo-generate/pull/365)
  - [chore(deps): bump structopt from 0.3.21 to 0.3.22](https://github.com/cargo-generate/cargo-generate/pull/364)
  - [chore(deps): bump assert_cmd from 1.0.5 to 1.0.7](https://github.com/cargo-generate/cargo-generate/pull/363)
  - [chore(clippy): make clippy happy](https://github.com/cargo-generate/cargo-generate/pull/361)
  - [chore(deps): bump openssl from 0.10.34 to 0.10.35](https://github.com/cargo-generate/cargo-generate/pull/358)
  - [chore(deps): bump cargo from 0.53.0 to 0.54.0](https://github.com/cargo-generate/cargo-generate/pull/357)
  - [chore(deps): bump anyhow from 1.0.40 to 1.0.41](https://github.com/cargo-generate/cargo-generate/pull/356)
  - [chore(deps): bump ignore from 0.4.17 to 0.4.18](https://github.com/cargo-generate/cargo-generate/pull/355)
  - [chore(deps): bump heck from 0.3.2 to 0.3.3](https://github.com/cargo-generate/cargo-generate/pull/354)
  - [chore(deps): bump assert_cmd from 1.0.4 to 1.0.5](https://github.com/cargo-generate/cargo-generate/pull/353)
  - [chore(deps): bump git2 from 0.13.19 to 0.13.20](https://github.com/cargo-generate/cargo-generate/pull/352)
  - [chore(deps): bump indicatif from 0.16.0 to 0.16.2](https://github.com/cargo-generate/cargo-generate/pull/351)
  - [chore(deps): bump thiserror from 1.0.24 to 1.0.25](https://github.com/cargo-generate/cargo-generate/pull/349)
  - [chore(docs): enhance vendored openssl installation](https://github.com/cargo-generate/cargo-generate/pull/347)
  - [chore(deps): bump assert_cmd from 1.0.3 to 1.0.4](https://github.com/cargo-generate/cargo-generate/pull/346)
  - [chore(deps): bump git2 from 0.13.18 to 0.13.19](https://github.com/cargo-generate/cargo-generate/pull/345)
  - [chore(deps): bump url from 2.2.1 to 2.2.2](https://github.com/cargo-generate/cargo-generate/pull/338)
  - [chore(deps): bump cargo from 0.52.0 to 0.53.0](https://github.com/cargo-generate/cargo-generate/pull/337)
  - [chore(deps): bump regex from 1.4.6 to 1.5.4](https://github.com/cargo-generate/cargo-generate/pull/336)
  - [chore(deps): bump openssl from 0.10.33 to 0.10.34](https://github.com/cargo-generate/cargo-generate/pull/335)
  - [chore(deps): bump predicates from 1.0.7 to 1.0.8](https://github.com/cargo-generate/cargo-generate/pull/334)
  - [chore(deps): bump regex from 1.4.5 to 1.4.6](https://github.com/cargo-generate/cargo-generate/pull/330)
  - [chore(deps): bump git2 from 0.13.17 to 0.13.18](https://github.com/cargo-generate/cargo-generate/pull/329)

### Contributors

- [@SkamDart](https://github.com/skamdart)
- [@chilipepperhott](https://github.com/chilipepperhott)
- [@sassman](https://github.com/sassman)
- [dependabot[bot]](https://github.com/apps/dependabot)

## [0.6.1] 2021-04-01

- ### 🛠️ Maintenance
  - [chore(deps): bump cargo from 0.51.0 to 0.52.0](https://github.com/cargo-generate/cargo-generate/pull/322)
  - [chore(deps): bump serde from 1.0.124 to 1.0.125](https://github.com/cargo-generate/cargo-generate/pull/321)
  - [chore(deps): bump walkdir from 2.3.1 to 2.3.2](https://github.com/cargo-generate/cargo-generate/pull/320)
  - [chore(deps): bump anyhow from 1.0.39 to 1.0.40](https://github.com/cargo-generate/cargo-generate/pull/319)
  - [chore(deps): bump anyhow from 1.0.38 to 1.0.39](https://github.com/cargo-generate/cargo-generate/pull/317)
  - [chore(deps): bump dialoguer from 0.7.1 to 0.8.0](https://github.com/cargo-generate/cargo-generate/pull/316)
  - [chore(deps): bump openssl from 0.10.32 to 0.10.33](https://github.com/cargo-generate/cargo-generate/pull/315)
  - [chore(deps): bump console from 0.14.0 to 0.14.1](https://github.com/cargo-generate/cargo-generate/pull/314)
  - [chore(deps): bump regex from 1.4.3 to 1.4.5](https://github.com/cargo-generate/cargo-generate/pull/313)
  - [chore(deps): bump remove_dir_all from 0.6.1 to 0.7.0](https://github.com/cargo-generate/cargo-generate/pull/311)
  - [chore(deps): bump liquid from 0.21.5 to 0.22.0](https://github.com/cargo-generate/cargo-generate/pull/305)

## [0.6.0] 2021-03-07

- ### ✨ Features
  - [interactive variable <enter> leads to default](https://github.com/cargo-generate/cargo-generate/pull/297),
    [issue/17](https://github.com/cargo-generate/cargo-generate/issues/17), by [sassman](https://github.com/sassman)

    This allows for lazy typing when using custom variables in templates, so that a user does not need to type the
    default value, but rather can press <enter> in order to accept the default value that is presented.

  - [Add `--vcs none` option to avoid initializing git repo](https://github.com/cargo-generate/cargo-generate/pull/293),
    [issue/244](https://github.com/cargo-generate/cargo-generate/issues/244), by [taurr](https://github.com/taurr)

  - [Add favorites on cargo-generate.toml user config file](https://github.com/cargo-generate/cargo-generate/pull/292),
    [issue/210](https://github.com/cargo-generate/cargo-generate/issues/210), by [taurr](https://github.com/taurr)

    This allows you to specify one or more shortcuts / favourites in your personal cargo-generate config file under
    `$CARGO_HOME/cargo-generate` or `$HOME/.cargo/cargo-generate`. You can read more
    about [this feature here](https://github.com/cargo-generate/cargo-generate#favorites)

    Update: [on backwards compatibility](https://github.com/cargo-generate/cargo-generate/pull/309)
    by [sassman](https://github.com/sassman)

  - [Add user specific template variables](https://github.com/cargo-generate/cargo-generate/pull/275),
    [issue/17](https://github.com/cargo-generate/cargo-generate/issues/17),
    by [pedrohjordao](https://github.com/pedrohjordao)

    This allows a template author to define template specific variables. Those variables can be of type string and bool.
    Further more they can be a choice of a provided list. You can read more
    about [this feature here in the docs](https://docs.rs/cargo-generate/0.6.0/cargo_generate/).

    A brief example:
    ```toml
    [placeholders]
    my-placeholder = { type = "string", prompt = "Hello?", choices = ["hello", "world"], default = "hello", regex = "*" }
    use-serde = { type = "bool", prompt = "Add serde support?", default = false }
    ```

- ### 🤕 Fixes
  - [dont init git when inside an existing repo](https://github.com/cargo-generate/cargo-generate/pull/290),
    [issue/244](https://github.com/cargo-generate/cargo-generate/issues/244), by [taurr](https://github.com/taurr)

- ### 🛠️ Maintenance
  - [Bump cargo from 0.50.1 to 0.51.0](https://github.com/cargo-generate/cargo-generate/pull/294),
    by [dependabot-preview[bot]](https://github.com/apps/dependabot-preview)

## [0.6.0-alpha.2] 2021-02-18 [PRERELEASED]

- ### ✨ Features
  - [interactive variable <enter> leads to default](https://github.com/cargo-generate/cargo-generate/pull/297),
    [issue/17](https://github.com/cargo-generate/cargo-generate/issues/17), by [sassman](https://github.com/sassman)

    This allows for lazy typing when using custom variables in templates, so that a user does not need to type the
    default value, but rather can press <enter> in order to accept the default value that is presented.

## [0.6.0-alpha.1] 2021-02-15 [PRERELEASED]

- ### 🛠️ Maintenance
  - [Bump cargo from 0.50.1 to 0.51.0](https://github.com/cargo-generate/cargo-generate/pull/294),
    by [dependabot-preview[bot]](https://github.com/apps/dependabot-preview)
- ### ✨ Features
  - [Add `--vcs none` option to avoid initializing git repo](https://github.com/cargo-generate/cargo-generate/pull/293),
    [issue/244](https://github.com/cargo-generate/cargo-generate/issues/244), by [taurr](https://github.com/taurr)
  - [Add favorites on cargo-generate.toml user config file](https://github.com/cargo-generate/cargo-generate/pull/292),
    [issue/210](https://github.com/cargo-generate/cargo-generate/issues/210), by [taurr](https://github.com/taurr)

    This allows you to specify one or more shortcuts / favourites in your personal cargo-generate config file under
    `$CARGO_HOME/cargo-generate` or `$HOME/.cargo/cargo-generate`. You can read more
    about [this feature here](https://github.com/cargo-generate/cargo-generate#favorites)

  - [Add user specific template variables](https://github.com/cargo-generate/cargo-generate/pull/275),
    [issue/17](https://github.com/cargo-generate/cargo-generate/issues/17),
    by [pedrohjordao](https://github.com/pedrohjordao)

    This allows a template author to define template specific variables. Those variables can be of type string and bool.
    Further more they can be a choice of a provided list. You can read more
    about [this feature here in the docs](https://docs.rs/cargo-generate/0.6.0/cargo_generate/).

    A brief example:
    ```toml
    [placeholders]
    my-placeholder = { type = "string", prompt = "Hello?", choices = ["hello", "world"], default = "hello", regex = "*" }
    use-serde = { type = "bool", prompt = "Add serde support?", default = false }
    ```

- ### 🤕 Fixes
  - [dont init git when inside an existing repo](https://github.com/cargo-generate/cargo-generate/pull/290),
    [issue/244](https://github.com/cargo-generate/cargo-generate/issues/244), by [taurr](https://github.com/taurr)

## [0.5.3] 2021-02-08

- ### 🛠️ Maintenance
  - **Bump [serde](https://github.com/serde-rs/serde) from 1.0.119 to
    1.0.123** [pull/287](https://github.com/cargo-generate/cargo-generate/pull/287) by @dependabot-preview
  - **Bump [liquid](https://github.com/cobalt-org/liquid-rust) from 0.21.4 to
    0.21.5** [pull/286](https://github.com/cargo-generate/cargo-generate/pull/286) by @dependabot-preview
  - **Bump [assert_cmd](https://github.com/assert-rs/assert_cmd) from 1.0.2 to
    1.0.3** [pull/285](https://github.com/cargo-generate/cargo-generate/pull/285) by @dependabot-preview
  - **Bump [cargo](https://github.com/rust-lang/cargo) from 0.50.0 to
    0.50.1** [pull/284](https://github.com/cargo-generate/cargo-generate/pull/284) by @dependabot-preview
  - **Bump [liquid-lib](https://github.com/cobalt-org/liquid-rust) from 0.21.1 to
    0.21.2** [pull/283](https://github.com/cargo-generate/cargo-generate/pull/283) by @dependabot-preview
  - **Bump [liquid-derive](https://github.com/cobalt-org/liquid-rust) from 0.21.0 to
    0.21.1** [pull/282](https://github.com/cargo-generate/cargo-generate/pull/282) by @dependabot-preview
  - **Bump [liquid-core](https://github.com/cobalt-org/liquid-rust) from 0.21.2 to
    0.21.3** [pull/281](https://github.com/cargo-generate/cargo-generate/pull/281) by @dependabot-preview

## [0.5.2] 2021-01-25

- ### ✨ Features
  - **make args fields public to provide a public API by [@No9], [pull/264]**

    this allows the external usage of `cargo-generate` from any lib / binary [see this example]

    [pull/264]: https://github.com/cargo-generate/cargo-generate/pull/264

    [@No9]: https://github.com/No9

    [see this example]: https://github.com/cargo-generate/cargo-generate/blob/161f320483e4276ab6d87a36b13a78f268239947/tests/integration/library.rs

  - **support operating system and architecture by [@macalimlim], [pull/252], [issues/251]**

    [@macalimlim]: https://github.com/macalimlim

    [issues/251]: https://github.com/cargo-generate/cargo-generate/issues/251

    [pull/252]: https://github.com/cargo-generate/cargo-generate/pull/252

- ### 🤕 Fixes
  - **fix creates an empty `.cargo-ok`, by [@thomcc], [pull/269], [issues/259]

    [@thomcc]: https://github.com/thomcc

    [pull/269]: https://github.com/cargo-generate/cargo-generate/pull/269

    [issues/259]: https://github.com/cargo-generate/cargo-generate/issues/259

  - **apply rust best practices clippy + fmt + ci/cd pipeline by [@sassman](https://github.com/sassman)
    , [pull/273](https://github.com/cargo-generate/cargo-generate/pull/273) [issue/270](https://github.com/cargo-generate/cargo-generate/issues/270)**

    Make clippy happy, and applies fmt for the whole code base Also, tests, linter (fmt+clippy) on mac, linux and
    windows are now executed on builds, means also for PRs Dismisses now travisCi and Appveyor

  - **handle default branch properly by [@cecton], [pull/263], [issues/258]**

    Make sure that not `master` or `main` as branch name is used and expected, but rather use the git default branch.

    [@cecton]: https://github.com/cecton

    [pull/263]: https://github.com/cargo-generate/cargo-generate/pull/263

    [issues/258]: https://github.com/cargo-generate/cargo-generate/issues/258

- ### 🛠️ Maintenance
  - **Shrink crate size by [@sassman](https://github.com/sassman)
    , [pull/272](https://github.com/cargo-generate/cargo-generate/pull/272)**

    Applies the suggestions of `cargo-diet` and hereby helps to reduce the crate size, by ship less irrelevant files.

- ### 👯 New Templates
  - **`godot-rust-template`: Create games with Godot and Rust by [@macalimlim], [pull/248]

    [@macalimlim]: https://github.com/macalimlim

    [pull/248]: https://github.com/cargo-generate/cargo-generate/pull/248

## 🕴️ 0.5.1

- ### 🤕 Fixes

  - **Ignore files in `.genignore` _before_ walking/substitution - [schell], [pull/235] [issue/236]**

    This fixes scenarios where liquid variables cause parsing errors in files that should be ignored.

    [schell]: https://github.com/schell

    [pull/235]: https://github.com/ashleygwilliams/cargo-generate/pull/235

    [issues/236]: https://github.com/ashleygwilliams/cargo-generate/issues/236

  - **Fix in CLI `help` option - [SirWindfield], [pull/234]**

    This fix the display of the `--branch` option in the help message, when executing `cargo generate --help`.

    [SirWindfield]: https://github.com/SirWindfield

    [pull/234]: https://github.com/ashleygwilliams/cargo-generate/pull/234

- ### 👯 New Templates

  - **`generust`: a template that provides a Rust web server and WASM client with
    some [interesting features](https://github.com/KyleU/generust/blob/main/doc/features.md) - [KyleU], [pull/203]**

    [KyleU]: https://github.com/KyleU

    [pull/203]: https://github.com/ashleygwilliams/cargo-generate/pull/203

  - **`orbtk`: a template that lets you create user interfaces using [OrbTk](https://github.com/redox-os/orbtk-template)
    - [FloVanGH], [pull/214]**

    [FloVanGH]: https://github.com/FloVanGH

    [pull/214]: https://github.com/ashleygwilliams/cargo-generate/pull/214

  - **`template-rust-backend-with-electron-frontend`: a template that lets you write a Rust native cdylib backend
    with [Electron](https://www.electronjs.org/) frontend - [usagi], [pull/218]**

    [usagi]: https://github.com/usagi

    [pull/218]: https://github.com/ashleygwilliams/cargo-generate/pull/218

  - **`swift-rust-xcode-template`: a template that lets you write an iOS app with [Swift](https://www.apple.com/swift/)
    and Rust - [simlay], [pull/219]**

    [simlay]: https://github.com/simlay

    [pull/219]: https://github.com/ashleygwilliams/cargo-generate/pull/219

  - **`Win32`: a template that provides an interface to write low-level Win32 applications in Rust - [ArmsOfSorrow]
    , [pull/220]**

    [ArmsOfSorrow]: https://github.com/ArmsOfSorrow

    [pull/220]: https://github.com/ashleygwilliams/cargo-generate/pull/220

  - **`QuickStart WebAssembly`: a template that lets you create a WebAssembly application with Rust
    - [PankajChaudhary5], [pull/227]**

    [PankajChaudhary5]: https://github.com/PankajChaudhary5

    [pull/227]: https://github.com/ashleygwilliams/cargo-generate/pull/227

  - **`rust-cli-template`: a template that lets you create easily a CLI with interesting features in
    it ([color_eyre](https://docs.rs/color_eyre), [tracing](https://docs.rs/tracing), in addition to benchmarking and
    testing boilerplate) - [9999years], [pull/239]**

    [9999years]: https://github.com/9999years

    [pull/239]: https://github.com/ashleygwilliams/cargo-generate/pull/239

  - **`mongodb-service-template`: a template that lets you create a GraphQL service with MongoDB as backing storage
    - [bdbmammoth], [pull/243]**

    [bdbmammoth]: https://github.com/bdbmammoth

    [pull/243]: https://github.com/ashleygwilliams/cargo-generate/pull/243

- ### 🛠️ Maintenance

  - **Support for `owner/repo` abbreviation for git URL format - [9999years], [pull/242]**

    [9999years]: https://github.com/9999years

    [pull/242]: https://github.com/ashleygwilliams/cargo-generate/pull/242

  - **Usage of default git branch instead of `master` - [9999years], [pull/241]**

    [9999years]: https://github.com/9999years

    [pull/241]: https://github.com/ashleygwilliams/cargo-generate/pull/241

  - **Updated `cargo-generate` to the latest and greatest in the Rust ecosystem - [Veetaha], [pull/237]**

    - Update all dependencies versions to latest ones
    - Replace `rustfmt-preview` with `rustfmt`
    - Replace `failure` with `anyhow`
    - Remove `quicli` completely
    - Update cargo authors copied code to latest cargo master
    - Fix clippy lints:
      - Replace &PathBuf to &Path in code
      - Remove bare `use crate_name` entires
      - Replace unexported `pub` with `pub(crate)`
    - Apply some cosmetic impl refactorings

    [Veetaha]: https://github.com/Veetaha

    [pull/237]: https://github.com/ashleygwilliams/cargo-generate/pull/237

  - **Code refactoring - [SirWindfield], [pull/233]**

    [SirWindfield]: https://github.com/SirWindfield

    [pull/233]: https://github.com/ashleygwilliams/cargo-generate/pull/233

  - **Support all `liquid` filters - [sassman], [pull/225]**

    - Upgrade `liquid` to v0.20
    - Enables all `liquid` default filters
    - Fix some findings of clippy

    [sassman]: https://github.com/sassman

    [pull/225]: https://github.com/ashleygwilliams/cargo-generate/pull/225

  - **Typo fixes in CONTRIBUTING.md - [Darrenmeehan], [pull/222]**

    [Darrenmeehan]: https://github.com/Darrenmeehan

    [pull/222]: https://github.com/ashleygwilliams/cargo-generate/pull/222

## 🧟‍♀️ 0.5.0

- ### ✨ Features

  - **Add a verbose flag for printing excluded files - [EverlastingBugstopper], [pull/199]**

    cargo-generate can now be run with a `--verbose` flag that will print the list of files/directories that it is
    ignoring. This means that by default the output for templates that exclude files will appear no differently from
    templates that do exclude files.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper

    [pull/199]: https://github.com/ashleygwilliams/cargo-generate/pull/199

- ### 🤕 Fixes

  - **Update two failure scenarios to exit with code 1 - [EverlastingBugstopper], [pull/198]**

    When cargo-generate fails due to an issue with git or if a target directory already exists, it will now fail with an
    exit code of 1.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper

    [pull/198]: https://github.com/ashleygwilliams/cargo-generate/pull/198

  - **Fix path display on Windows - [tommyshem], [issue/192] [pull/195]**

    Changes Windows file output from `"C:\\Users\\Me\\src\\example-com\\.genignore"`
    to `"C:\Users\Me\src\example-com\.genignore"`

    [tommyshem]: https://github.com/tommyshem

    [pull/195]: https://github.com/ashleygwilliams/cargo-generate/pull/195

    [issue/192]: https://github.com/ashleygwilliams/cargo-generate/issues/192

  - **Don't remove .github directory when only .git should be removed - [softprops], [issue/190] [pull/191]**

    When generating a new project, cargo-generate will remove the `.git` directory and initialize a new git tree. In
    doing so, it would remove any file or directory containing `.git`, including `.github`. This release fixes that bug!

    [softprops]: https://github.com/softprops

    [pull/191]: https://github.com/ashleygwilliams/cargo-generate/pull/191

    [issue/190]: https://github.com/ashleygwilliams/cargo-generate/issues/190

- ### 🛠️ Maintenance

  - **Fix a dead link in TEMPLATES.md - [yaahc], [pull/186]**

    [yaahc]: https://github.com/yaahc

    [pull/186]: https://github.com/ashleygwilliams/cargo-generate/pull/186

  - **Use Cargo.lock when compiling binary - [xortive], [pull/188]**

  [xortive]: https://github.com/xortive

  [pull/188]: https://github.com/ashleygwilliams/cargo-generate/pull/188

## 🍕 0.4.0

- ### ✨ Features

  - **Add config file for configuring include / exclude of files to template - [xortive], [pull/174]**

    Adds support for
    the [cargo-generate.toml](https://github.com/ashleygwilliams/cargo-generate/blob/main/README.md#include--exclude)
    file, which allows templates to configure which files should be processed, either using a whitelist
    method (`include`), or a blacklist method (`exclude`). When both `include` and `exclude` are present, `include`
    will be preferred. This
    mirrors [similar `include`/`exclude` behavior in `cargo`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields-optional)
    , and uses some of the same code.

    #### Include List

      ```toml
      [template]
      include = ["Cargo.toml", "src/lib.rs"] # Only search for and replace template tags in Cargo.toml and src/lib.rs
      ```

    #### Exclude List

      ```toml
      [template]
      exclude = ["public/image.js"] # Don't look for template tags in public/image.js
      ```

    #### Invalid Configuration

      ```toml
      [template]
      # This is an "invalid" configuration, so cargo-generate takes the "more specific" include option
      # and will only search for and replace template tags in Cargo.toml.
      include = ["Cargo.toml"]
      exclude = ["public/image.js"]
      ```

    [xortive]: https://github.com/xortive

    [pull/174]: https://github.com/ashleygwilliams/cargo-generate/pull/174

- ### 🤕 Fixes

  - **Respect default branch name of template - [RotationMatrix], [pull/166]**

    The `--branch` flag will now work as intended and set the initial HEAD to the specified branch name. For example,
    running `cargo generate --git <gh pages template> --branch gh-pages` will set your generated project's default
    branch to `gh-pages`.

    [RotationMatrix]: https://github.com/RotationMatrix

    [pull/166]: https://github.com/ashleygwilliams/cargo-generate/pull/166

- ### 🛠️ Maintenance

  - **Cleanup of lingering clippy / rustfmt warnings - [ashleygwilliams], [pull/175]**

    [ashleygwilliams]: https://github.com/ashleygwilliams

    [pull/175]: https://github.com/ashleygwilliams/cargo-generate/pull/175

  - **Fix assert! macro usage - [rasendubi], [pull/157]**

    [rasendubi]: https://github.com/rasendubi

    [pull/157]: https://github.com/ashleygwilliams/cargo-generate/pull/157

## 🛠 0.3.1

- ### 🤕 Fixes

  - **Fix messages related to rename behavior - [xortive], [pull/162]**

    `--force` would stop `cargo-generate` renaming your project, but we would still message that the rename was
    happening. Not anymore!

    [xortive]: https://github.com/xortive

    [pull/162]: https://github.com/ashleygwilliams/cargo-generate/pull/162

  - **Use Vendored OpenSSL for macOS - [xortive], [pull/169]**

    Our prebuilt binaries for macOS were using dynamically linked OpenSSL, and our CI was dynamically linking to a
    version of OpenSSL not installed by default on most macs. Now, the macOS release build of `cargo-generate`
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

- ### 🛠️ Maintenance

  - **Update to liquid 0.19 - [epage], [pull/165]**

    [epage]: https://github.com/epage

    [pull/165]: https://github.com/ashleygwilliams/cargo-generate/pull/165

## ⭐ 0.3.0

- ### ✨ Features

  - **Support case filters in templates - [epage], [issue/117] [pull/140]**

    Because we leverage `liquid` as a templating engine under the hood, we can add some specific filters to our
    substitutions to add a little more awesome. This feature adds 4 filters:

    - `capitalize`
    - `pascal_case`
    - `kebab_case`
    - `snake_case`

    Now we can tame any set of letters with any type of capitalization or dash! Give them a try and let us know what
    other types of features you'd like to see.

    [issue/117]: https://github.com/ashleygwilliams/cargo-generate/issues/117

    [pull/140]: https://github.com/ashleygwilliams/cargo-generate/pull/140

- ### 🤕 Fixes

  - **Windows releases on Appveyor - [jaysonsantos], [issue/145] [pull/146]**

    Thanks to a new tool, [`wrangler`], that uses `cargo-generate` as a dependency, it was discovered that we were
    shipping broken Windows binaries. We've fixed that now!

    [`wrangler`]: https://github.com/ashleygwilliams/cargo-generate

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

    When we previously merged the PR in 0.2.0 that leveraged `cargo` to clone the templates, enabling folks to work with
    private repositories- we introduced a `GitConfig::new` function
    (replacing work done previously by `libgit2`). This function works great- but did not support relative paths. We
    didn't catch this because we weren't testing the relative paths usecase!

    With this PR, [DD5HT] has restored the relative path functionality- and added a test to prevent further regressions
    of this function!

    [issue/128]: https://github.com/ashleygwilliams/cargo-generate/issues/128

    [pull/129]: https://github.com/ashleygwilliams/cargo-generate/pull/129

- ### 🛠️ Maintenance

  - **cargo update/cargo fmt - [ashleygwilliams], [pull/134] [pull/133]**

    [pull/133]: https://github.com/ashleygwilliams/cargo-generate/pull/133

    [pull/134]: https://github.com/ashleygwilliams/cargo-generate/pull/134

## 🌠 0.2.1

- ### 🤕 Fixes

  - **don't error on missing `.genignore` file - [DD5HT], [issue/116] [pull/120]**

    With 0.2.0 we introduced the idea of a `.genignore` file, however, we didn't account the situation where one would
    not be present. Thanks for filing [an issue][issue/116]
    [Diggsey] and thanks for the quick fix [DD5HT]!

    [issue/116]: https://github.com/ashleygwilliams/cargo-generate/issues/116

    [pull/120]: https://github.com/ashleygwilliams/cargo-generate/pull/120

    [Diggsey]: https://github.com/Diggsey

  - **enable use on private repositories- [ChristopherMacGown], [pull/119]**

    This PR leveraged `cargo` to enable the ability to pull authenticated repos. As this project grows into something
    we'd like to integrate into `cargo`, this gives us greater functionality and also moves us closer to `cargo`'s
    codebase. Yay!

    [ChristopherMacGown]: https://github.com/ChristopherMacGown

    [pull/119]: https://github.com/ashleygwilliams/cargo-generate/pull/119

  - **exclude submodules git metadata - [ChristopherMacGown], [pull/119]**

    Two bugs, one PR! Adding a test found that git metadata wasn't being excluded from submodules- now it is! Thanks so
    much!

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

    Motivated by a desire to more easily distributed the project - we now build binaries for our releases. No more
    waiting for compilation! You can just download and go!

    [issue/99]: https://github.com/ashleygwilliams/cargo-generate/issues/99

    [pull/111]: https://github.com/ashleygwilliams/cargo-generate/pull/111

    [pull/112]: https://github.com/ashleygwilliams/cargo-generate/pull/112

  - **Allow Liquid Templating `date` filter - [ashleygwilliams], [issue/70] [pull/106]**

    By request, we've turned on the `date` filter for our templates. Now you can add nicely formatted dates to your
    projects! For more information, check out the
    [Liquid `date` filter documentation].

    [Liquid `date` filter documentation]: https://shopify.github.io/liquid/filters/date/

    [issue/70]: https://github.com/ashleygwilliams/cargo-generate/issues/70

    [pull/106]: https://github.com/ashleygwilliams/cargo-generate/pull/106

  - **Add `.genignore`, ability to ignore files - [DD5HT], [issue/82] [pull/96]**

    You can now add a `.genignore` file to your template. This file will specify the files to be "cleaned up" or "
    removed" from the template once it has been cloned to the user's local machine.

    [issue/82]: https://github.com/ashleygwilliams/cargo-generate/issues/82

    [pull/96]: https://github.com/ashleygwilliams/cargo-generate/pull/96

  - **Add `--branch` for specifying a branch - [posborne], [issue/71] [pull/94]**

    We originally had no way to specify a git template on a per branch basis, opting to only support the primary branch.
    Now you can specify a branch:

    ```
    cargo generate --git <gitURL> --branch <branchname>
    ```

    [posborne]: https://github.com/posborne

    [issue/71]: https://github.com/ashleygwilliams/cargo-generate/issues/71

    [pull/94]: https://github.com/ashleygwilliams/cargo-generate/pull/94

  - **Warn user if we change project name casing - [k0pernicus], [issue/65] [pull/84]**

    `cargo-generate` will automagically "fix" the casing of your project name to match Cargo standards. If we end up
    changing the name you provide- we'll warn to let you know!

    [k0pernicus]: https://github.com/k0pernicus

    [issue/65]: https://github.com/ashleygwilliams/cargo-generate/issues/65

    [pull/84]: https://github.com/ashleygwilliams/cargo-generate/pull/84

  - **Add `--force` flag to skip casing check on project name - [toVersus], [issue/66] [pull/69]**

    `cargo-generate` will automagically "fix" the casing of your project name to match Cargo standards. If you'd like to
    skip that, you can add `--force`.

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

    Previous to this commit, `cargo-generate` was a CLI tool that was invoked by the command `cargo-generate` (note the
    dash). However, this tool intends to be a cargo subcommand! This commit changes how you invoke the tool- no more
    dash!

    ```
    cargo generate --git https://github.com/username/project --name look-ma-no-dash
    ```

    [csmoe]: https://github.com/csmoe

    [issue/39]: https://github.com/ashleygwilliams/cargo-generate/issues/39

    [pull/44]: https://github.com/ashleygwilliams/cargo-generate/pull/44

  - **Fix casing on `crate_name` substitution - [ashleygwilliams], [issue/41] [pull/56]**

    `crate_name` substitution is supposed to be a convenience, converting the given project's name to a name that you
    could use with `extern crate` or in other *in-code*
    situations. Just one problem- before this commit, it didn't change the case!
    Now it will. Thanks so much to [fitzgen] for filing this issue (and a bunch of others)!

    [ashleygwilliams]: https://github.com/ashleygwilliams

    [issue/41]: https://github.com/ashleygwilliams/cargo-generate/issues/41

    [pull/56]: https://github.com/ashleygwilliams/cargo-generate/pull/56

    [fitzgen]: https://github.com/fitzgen

- ### 📖 Documentation

  - **Document build and runtime dependencies - [migerh], [issue/42] [pull/45]**

    There are a few dependencies for the project that we hadn't documented. Many folks have these already installed, but
    some don't- so it's great that they are now well documented in the `README`.

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

[0.10.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.9.0...v0.10.0

[0.9.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.8.0...v0.9.0

[0.8.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.7.2...v0.8.0

[0.7.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.7.1...v0.7.2

[0.7.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.7.0...v0.7.1

[0.7.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.6.1...v0.7.0

[0.6.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.6.0...v0.6.1

[0.6.0]: https://github.com/cargo-generate/cargo-generate/compare/v0.6.0-alpha.2...v0.6.0

[0.6.0-alpha.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.6.0-alpha.1...v0.6.0-alpha.2

[0.6.0-alpha.1]: https://github.com/cargo-generate/cargo-generate/compare/v0.5.3...v0.6.0-alpha.1

[0.5.3]: https://github.com/cargo-generate/cargo-generate/compare/v0.5.2...v0.5.3

[0.5.2]: https://github.com/cargo-generate/cargo-generate/compare/v0.5.1...v0.5.2
