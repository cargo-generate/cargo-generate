use crate::interactive::LIST_SEP_PRETTY;

pub trait ArrayToTemplateString {
    fn render_as_string(&self) -> String;
}

impl ArrayToTemplateString for Vec<String> {
    fn render_as_string(&self) -> String {
        format!(
            "[{}]",
            self.into_iter()
                .map(|item| format!(r#""{item}""#))
                .collect::<Vec<String>>()
                .join(LIST_SEP_PRETTY)
        )
    }
}
