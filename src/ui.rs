//! Thin facade around status-style output.
//!
//! Two backends are available:
//! - `classic` (default): plain println/eprintln with `console` styling.
//! - `next` (behind the `ui-next` feature): [`cliclack`] structured output.
//!
//! Public entry points keep identical signatures so call-sites don't branch.

use anyhow::Result;

#[cfg(not(feature = "ui-next"))]
mod classic {
    use super::Result;
    use crate::emoji;
    use console::style;

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
        eprintln!("{} {}", emoji::DIAMOND, msg.as_ref());
        Ok(())
    }

    pub fn warning(msg: impl AsRef<str>) -> Result<()> {
        eprintln!("{} {}", emoji::WARN, style(msg.as_ref()).yellow());
        Ok(())
    }

    pub fn note(title: impl AsRef<str>, content: impl AsRef<str>) -> Result<()> {
        eprintln!(
            "\n{} {}\n{}\n",
            emoji::DIAMOND,
            style(title.as_ref()).bold().cyan(),
            content.as_ref()
        );
        Ok(())
    }
}

#[cfg(feature = "ui-next")]
mod next {
    use super::Result;

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
}

#[cfg(not(feature = "ui-next"))]
pub use classic::*;
#[cfg(feature = "ui-next")]
pub use next::*;
