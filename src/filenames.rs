use crate::Result;

use liquid::{Object, Parser};
use std::path::{Path, PathBuf};

pub fn substitute_filename(filepath: &Path, parser: &Parser, context: &Object) -> Result<PathBuf> {
    let filename = filepath.file_name().and_then(|s| s.to_str()).unwrap();
    let path = filepath.parent().unwrap_or(Path::new(""));

    let parsed_filename = parser.parse(filename)?.render(context)?;

    let options = sanitize_filename::Options {
        truncate: true,   // true by default, truncates to 255 bytes
        replacement: "_", // str to replace sanitized chars/strings
        ..sanitize_filename::Options::default()
    };
    let sanitized = sanitize_filename::sanitize_with_options(parsed_filename, options);

    Ok(path.join(Path::new(sanitized.as_str())))
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
        assert_eq!(
            substitute_filename("/tmp/project/{{author}}.rs", prepare_context("sassman")).unwrap(),
            "/tmp/project/sassman.rs"
        );
    }

    #[test]
    fn should_prevent_invalid_filenames() {
        assert_eq!(
            substitute_filename("/tmp/project/{{author}}.rs", prepare_context("s/a/s")).unwrap(),
            "/tmp/project/s_a_s.rs"
        );
    }

    #[test]
    fn should_prevent_exploitation() {
        assert_eq!(
            substitute_filename(
                "/tmp/project/{{author}}.rs",
                prepare_context("../../etc/passwd")
            )
            .unwrap(),
            "/tmp/project/.._.._etc_passwd.rs"
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
