//! Thin facade around user-facing output and prompts.
//!
//! Two backends are available:
//! - `ui_classic` (default): dialoguer + plain println/eprintln with `console` styling.
//! - `ui_next` (behind the `ui-next` feature): [`cliclack`] structured output.
//!
//! Public entry points keep identical signatures so call-sites don't branch.

use anyhow::Result;

/// Optional inline validator for text input widgets.
///
/// Returns `Ok(())` to accept the value, `Err(msg)` to reject and re-prompt
/// with `msg` shown to the user.
pub type Validator = Box<dyn Fn(&str) -> std::result::Result<(), String> + 'static>;

#[cfg(not(feature = "ui-next"))]
mod ui_classic {
    use super::{Result, Validator};
    use crate::emoji;
    use console::style;
    use dialoguer::{theme::ColorfulTheme, Confirm, Editor, Input, MultiSelect, Select};

    // -- status output --

    pub fn intro(_msg: impl AsRef<str>) -> Result<()> {
        // classic UI historically has no banner at startup
        Ok(())
    }

    pub fn outro(msg: impl AsRef<str>) -> Result<()> {
        eprintln!("{} {}", emoji::SPARKLE, style(msg.as_ref()).bold().green());
        Ok(())
    }

    pub fn outro_cancel(msg: impl AsRef<str>) -> Result<()> {
        eprintln!("{} {}", emoji::ERROR, style(msg.as_ref()).bold().red());
        Ok(())
    }

    pub fn info(msg: impl AsRef<str>) -> Result<()> {
        eprintln!("{} {}", emoji::INFO, msg.as_ref());
        Ok(())
    }

    pub fn warning(msg: impl AsRef<str>) -> Result<()> {
        eprintln!("{} {}", emoji::INFO, style(msg.as_ref()).yellow());
        Ok(())
    }

    pub fn note(title: impl AsRef<str>, content: impl AsRef<str>) -> Result<()> {
        eprintln!(
            "\n{} {}\n{}\n",
            emoji::INFO,
            style(title.as_ref()).bold().cyan(),
            content.as_ref()
        );
        Ok(())
    }

    // -- prompt widgets --

    fn styled_prompt(prompt: &str, default: Option<&str>) -> String {
        let base = format!("{} {}", emoji::SHRUG, style(prompt).bold());
        match default {
            Some(d) => format!("{base} [default: {}]", style(d).bold()),
            None => base,
        }
    }

    pub fn input(prompt: &str, default: Option<&str>, validate: Option<Validator>) -> Result<String> {
        let mut input = Input::<String>::new().with_prompt(styled_prompt(prompt, default));
        if let Some(s) = default {
            input = input.default(s.to_owned());
        }
        if let Some(v) = validate {
            input = input.validate_with(move |s: &String| -> std::result::Result<(), String> {
                v(s.as_str())
            });
        }
        Ok(input.interact()?)
    }

    pub fn select(prompt: &str, choices: &[String], initial: usize) -> Result<String> {
        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(styled_prompt(prompt, None))
            .items(choices)
            .default(initial.min(choices.len().saturating_sub(1)))
            .interact()?;
        Ok(choices[idx].clone())
    }

    pub fn multiselect(
        prompt: &str,
        choices: &[String],
        initial_flags: &[bool],
    ) -> Result<Vec<String>> {
        let flags: Vec<bool> = if initial_flags.len() == choices.len() {
            initial_flags.to_vec()
        } else {
            vec![false; choices.len()]
        };
        let indices = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(styled_prompt(prompt, None))
            .items(choices)
            .defaults(&flags)
            .interact()?;
        Ok(indices.into_iter().map(|i| choices[i].clone()).collect())
    }

    pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
        Ok(Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(styled_prompt(prompt, None))
            .default(default)
            .interact()?)
    }

    pub fn editor(initial: &str) -> Result<Option<String>> {
        Ok(Editor::new().edit(initial)?)
    }
}

#[cfg(feature = "ui-next")]
mod ui_next {
    use super::{Result, Validator};

    // -- status output --

    pub fn intro(msg: impl AsRef<str>) -> Result<()> {
        cliclack::intro(msg.as_ref())?;
        Ok(())
    }

    pub fn outro(msg: impl AsRef<str>) -> Result<()> {
        cliclack::outro(msg.as_ref())?;
        Ok(())
    }

    pub fn outro_cancel(msg: impl AsRef<str>) -> Result<()> {
        cliclack::outro_cancel(msg.as_ref())?;
        Ok(())
    }

    pub fn info(msg: impl AsRef<str>) -> Result<()> {
        cliclack::log::info(msg.as_ref())?;
        Ok(())
    }

    pub fn warning(msg: impl AsRef<str>) -> Result<()> {
        cliclack::log::warning(msg.as_ref())?;
        Ok(())
    }

    pub fn note(title: impl AsRef<str>, content: impl AsRef<str>) -> Result<()> {
        cliclack::note(title.as_ref(), content.as_ref())?;
        Ok(())
    }

    // -- prompt widgets --

    pub fn input(prompt: &str, default: Option<&str>, validate: Option<Validator>) -> Result<String> {
        let mut input = cliclack::input(prompt);
        if let Some(s) = default {
            input = input.default_input(s).placeholder(s);
        }
        if let Some(v) = validate {
            input = input.validate(move |s: &String| v(s.as_str()));
        }
        let result: String = input.interact()?;
        Ok(result)
    }

    pub fn select(prompt: &str, choices: &[String], initial: usize) -> Result<String> {
        let initial = choices.get(initial).cloned().unwrap_or_else(|| {
            choices.first().cloned().unwrap_or_default()
        });
        let mut select = cliclack::select(prompt);
        for choice in choices {
            let hint = if *choice == initial { "default" } else { "" };
            select = select.item(choice.clone(), choice, hint);
        }
        select = select.initial_value(initial);
        let chosen: String = select.interact()?;
        Ok(chosen)
    }

    pub fn multiselect(
        prompt: &str,
        choices: &[String],
        initial_flags: &[bool],
    ) -> Result<Vec<String>> {
        let mut ms = cliclack::multiselect(prompt).required(false);
        let initial: Vec<String> = choices
            .iter()
            .enumerate()
            .filter(|(i, _)| initial_flags.get(*i).copied().unwrap_or(false))
            .map(|(_, c)| c.clone())
            .collect();
        for choice in choices {
            let hint = if initial.contains(choice) { "default" } else { "" };
            ms = ms.item(choice.clone(), choice, hint);
        }
        if !initial.is_empty() {
            ms = ms.initial_values(initial);
        }
        let chosen: Vec<String> = ms.interact()?;
        Ok(chosen)
    }

    pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
        let chosen: bool = cliclack::confirm(prompt).initial_value(default).interact()?;
        Ok(chosen)
    }

    pub fn editor(initial: &str) -> Result<Option<String>> {
        use std::io::Write;
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "notepad".to_string()
                } else {
                    "vi".to_string()
                }
            });

        let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile()?;
        write!(tmp, "{}", initial)?;
        tmp.flush()?;

        let path = tmp.path().to_owned();
        let status = std::process::Command::new(&editor).arg(&path).status()?;

        if !status.success() {
            return Ok(None);
        }

        let content = fs_err::read_to_string(&path)?;
        if content == initial {
            Ok(None)
        } else {
            Ok(Some(content))
        }
    }
}

#[cfg(not(feature = "ui-next"))]
pub use ui_classic::*;
#[cfg(feature = "ui-next")]
pub use ui_next::*;
