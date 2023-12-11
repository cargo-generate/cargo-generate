# Template Authoring

> Available since version [0.9.0](https://github.com/cargo-generate/cargo-generate/releases/tag/v0.9.0)

As a template author you're probably concerned about successful builds of your template?

Imagine a couple of months after your first template release, some new versions of any dependencies would break your template, and you would not even be aware of it?

The answer to this question is a vital build pipeline for your template project. This challenge got now much simpler to solve with the new official [cargo-generate GitHub Action](https://github.com/marketplace/actions/cargo-generate).

Here an example:

```sh
tree .github
.github
└── workflows
    └── build.yml
```

The content of `build.yml` as a paste template:

```yaml
name: Build Template
on:
  # https://docs.github.com/en/actions/reference/events-that-trigger-workflows#workflow_dispatch
  workflow_dispatch:
  schedule:
    - cron: '0 18 * * 5'
  push:
    branches: [ '*' ]
    paths-ignore:
      - "**/docs/**"
      - "**.md"

jobs:
  build:
    runs-on: ubuntu-latest
    env:
      PROJECT_NAME: mytemplate
    steps:
      - uses: actions/checkout@v4
      - uses: cargo-generate/cargo-generate-action@v0.17
        with:
          name: ${{ env.PROJECT_NAME }}
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      # we need to move the generated project to a temp folder, away from the template project
      # otherwise `cargo` runs would fail 
      # see https://github.com/rust-lang/cargo/issues/9922
      - run: |
          mv $PROJECT_NAME ${{ runner.temp }}/
          cd ${{ runner.temp }}/$PROJECT_NAME
          cargo check
```

This is a very simple pipeline that builds weekly and on push.
It processes your template repo and runs a `cargo check` as the final step. That's it, a good start to build on.
