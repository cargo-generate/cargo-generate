# Common Pitfalls

When creating templates with `cargo-generate`, there are several common issues and pitfalls that template authors may encounter. This section aims to highlight these issues and provide guidance on how to avoid them.

## GitHub Actions and Liquid Template Language Interference

GitHub Actions use their own templating language, which can interfere with the Liquid template language used by `cargo-generate`. This can lead to unexpected behavior when placeholders are used in GitHub Actions workflow files.

### Issue

When using placeholders in GitHub Actions workflow files, the syntax for GitHub Actions (`${{ ... }}`) can conflict with the Liquid syntax (`{{ ... }}`). This can cause errors or unexpected behavior during template generation.

For more details, you can refer to the [discussion](https://github.com/cargo-generate/cargo-generate/discussions/843) that was opened for this purpose.

### Workarounds

1. **Escape Liquid Syntax**: One way to avoid conflicts is to escape the Liquid syntax in the workflow files. This can be done by using `{% raw %}` and `{% endraw %}` tags around the GitHub Actions syntax.

    ```yaml
    jobs:
      build:
        runs-on: ubuntu-latest
        steps:
          - name: Checkout code
            uses: actions/checkout@v2
          - name: Run cargo-generate
            run: |
              cargo generate --git https://github.com/your/repo.git --name ${{ '{% raw %}' }}{{ project-name }}{% endraw %}
    ```

2. **Use Different Placeholders**: Another approach is to use different placeholders for GitHub Actions and Liquid. For example, you can use a different syntax for placeholders in GitHub Actions and then replace them with the correct values in a pre-processing step.

    ```yaml
    jobs:
      build:
        runs-on: ubuntu-latest
        steps:
          - name: Checkout code
            uses: actions/checkout@v2
          - name: Set up project
            run: |
              PROJECT_NAME={{ project-name }}
              echo "Project name is $PROJECT_NAME"
    ```

3. **Use `cargo-generate` Placeholders Sparingly**: Limit the use of `cargo-generate` placeholders in GitHub Actions workflow files to only where necessary. This reduces the chances of conflicts and makes the workflow files easier to manage.

4. **Liquid Prepend and Append**: Use Liquid's `prepend` and `append` filters to dynamically generate the GitHub Actions syntax. This ensures that the placeholders are correctly processed by Liquid and result in the correct GitHub Actions markup.

    ```yaml
    jobs:
      build:
        runs-on: ubuntu-latest
        steps:
          - name: Checkout code
            uses: actions/checkout@v2
          - name: Set up project
            run: |
              echo "${{ "github-variable" | prepend: "{{" | append: "}}" }}"
    ```

5. **Pre-Hook Rhai Script**: Another solution is to create a pre-hook Rhai script that executes `gsed` or `sed` to replace all GitHub Actions placeholders with the Liquid syntax on the fly. This automates the escaping process.

    ```toml
    [hooks]
    pre = ["pre-hook.rhai"]
    ```

    ```rhai
    // pre-hook.rhai
    let result = system::command("gsed", ["-i", 's/${{ /${{ "{{" | prepend: "{{" | append: "}}" }}/g', "path/to/workflow/file.yml"]);
    if result != 0 {
        abort("Failed to replace GitHub Actions placeholders");
    }
    ```

    For more details, you can refer to the [issue](https://github.com/cargo-generate/cargo-generate/issues/1387#issuecomment-2691737440) that was opened for this.

## Undefined Placeholders

Another common pitfall is using placeholders that are not defined. When a placeholder is not defined, `cargo-generate` will not throw an error; instead, it will replace the placeholder with an empty string. This can lead to unexpected results in the generated files.

### Issue

If a placeholder is used in a template file but is not defined in the `cargo-generate.toml` file or provided by the user, it will be replaced with an empty string. This can cause issues such as missing values in configuration files or broken code.

### Solution

1. **Define All Placeholders**: Ensure that all placeholders used in the template files are defined in the `cargo-generate.toml` file. This includes providing default values or prompting the user for input.

    ```toml
    [placeholders]
    project_name = { prompt = "Enter project name", default = "my_project", type = "string" }
    author_name = { prompt = "Enter author name", default = "John Doe", type = "string" }
    ```

2. **Validate Placeholder Usage**: Before generating the template, validate that all placeholders used in the template files are defined. This can be done by reviewing the template files and cross-referencing them with the placeholders defined in the `cargo-generate.toml` file.

3. **Provide Default Values**: Where possible, provide default values for placeholders to ensure that they are always replaced with meaningful values.

    ```toml
    [placeholders]
    project_name = { prompt = "Enter project name", default = "my_project", type = "string" }
    ```

By being aware of these common issues and pitfalls, template authors can create more robust and reliable templates with `cargo-generate`. Proper handling of GitHub Actions and Liquid template language interference, as well as ensuring that all placeholders are defined, will help avoid unexpected behavior and improve the overall template generation experience.
