use crate::Result;

use crate::template::render_string_gracefully;
use liquid::{Object, Parser};
use std::path::{Component, Path, PathBuf};

pub fn substitute_filename(filepath: &Path, parser: &Parser, context: &Object) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    for elem in filepath.components() {
        match elem {
            Component::Normal(e) => {
                let parsed = render_string_gracefully(context, parser, e.to_str().unwrap())?;
                let parsed = sanitize_filename(parsed.as_str());
                path.push(parsed);
            }
            other => path.push(other),
        }
    }
    Ok(path)
}

fn sanitize_filename(filename: &str) -> String {
    use sanitize_filename::sanitize_with_options;

    let options = sanitize_filename::Options {
        truncate: true,   // true by default, truncates to 255 bytes
        replacement: "_", // str to replace sanitized chars/strings
        ..sanitize_filename::Options::default()
    };

    sanitize_with_options(filename, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use liquid::model::Value;

    #[test]
    fn should_do_happy_path() {
        assert_eq!(
            substitute_filename("{{author}}.rs", prepare_context("sassman")).unwrap(),
            "sassman.rs"
        );
        #[cfg(unix)]
        assert_eq!(
            substitute_filename("/tmp/project/{{author}}.rs", prepare_context("sassman")).unwrap(),
            "/tmp/project/sassman.rs"
        );
        #[cfg(unix)]
        assert_eq!(
            substitute_filename(
                "/tmp/project/{{author}}/{{author}}.rs",
                prepare_context("sassman")
            )
            .unwrap(),
            "/tmp/project/sassman/sassman.rs"
        );
        #[cfg(windows)]
        assert_eq!(
            substitute_filename(
                "C:\\tmp\\project\\{{author}}.rs",
                prepare_context("sassman")
            )
            .unwrap(),
            "C:\\tmp\\project\\sassman.rs"
        );
        #[cfg(windows)]
        assert_eq!(
            substitute_filename(
                "C:\\tmp\\project\\{{author}}\\{{author}}.rs",
                prepare_context("sassman")
            )
            .unwrap(),
            "C:\\tmp\\project\\sassman\\sassman.rs"
        );
    }

    #[test]
    fn should_prevent_invalid_filenames() {
        #[cfg(unix)]
        assert_eq!(
            substitute_filename("/tmp/project/{{author}}.rs", prepare_context("s/a/s")).unwrap(),
            "/tmp/project/s_a_s.rs"
        );
        #[cfg(unix)]
        assert_eq!(
            substitute_filename(
                "/tmp/project/{{author}}/{{author}}.rs",
                prepare_context("s/a/s")
            )
            .unwrap(),
            "/tmp/project/s_a_s/s_a_s.rs"
        );
        #[cfg(windows)]
        assert_eq!(
            substitute_filename(
                "C:\\tmp\\project\\{{author}}.rs",
                prepare_context("s\\a\\s")
            )
            .unwrap(),
            "C:\\tmp\\project\\s_a_s.rs"
        );
        #[cfg(windows)]
        assert_eq!(
            substitute_filename(
                "C:\\tmp\\project\\{{author}}\\{{author}}.rs",
                prepare_context("s\\a\\s")
            )
            .unwrap(),
            "C:\\tmp\\project\\s_a_s\\s_a_s.rs"
        );
    }

    #[test]
    fn should_prevent_exploitation() {
        #[cfg(unix)]
        assert_eq!(
            substitute_filename(
                "/tmp/project/{{author}}.rs",
                prepare_context("../../etc/passwd")
            )
            .unwrap(),
            "/tmp/project/.._.._etc_passwd.rs"
        );
        #[cfg(unix)]
        assert_eq!(
            substitute_filename(
                "/tmp/project/{{author}}/main.rs",
                prepare_context("../../etc/passwd")
            )
            .unwrap(),
            "/tmp/project/.._.._etc_passwd/main.rs"
        );
        #[cfg(windows)]
        assert_eq!(
            substitute_filename(
                "C:\\tmp\\project\\{{author}}.rs",
                prepare_context("..\\..\\etc\\passwd")
            )
            .unwrap(),
            "C:\\tmp\\project\\.._.._etc_passwd.rs"
        );
        #[cfg(windows)]
        assert_eq!(
            substitute_filename(
                "C:\\tmp\\project\\{{author}}\\main.rs",
                prepare_context("..\\..\\etc\\passwd")
            )
            .unwrap(),
            "C:\\tmp\\project\\.._.._etc_passwd\\main.rs"
        );
    }

    //region wrapper helpers
    fn prepare_context(value: &str) -> Object {
        let mut ctx = Object::default();
        ctx.entry("author")
            .or_insert(Value::scalar(value.to_string()));

        ctx
    }

    fn substitute_filename(f: &str, ctx: Object) -> Result<String> {
        let parser = Parser::default();

        super::substitute_filename(f.as_ref(), &parser, &ctx)
            .map(|p| p.to_str().unwrap().to_string())
    }
    //endregion
}
