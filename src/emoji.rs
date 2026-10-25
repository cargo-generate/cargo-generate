use console::Emoji;

pub static ERROR: Emoji<'_, '_> = Emoji("⛔  ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", "");
pub static WARN: Emoji<'_, '_> = Emoji("⚠️  ", "");
#[cfg(feature = "git")]
pub static WRENCH: Emoji<'_, '_> = Emoji("🔧  ", "");
pub static SHRUG: Emoji<'_, '_> = Emoji("🤷  ", "");
pub static DIAMOND: Emoji<'_, '_> = Emoji("🔸  ", "");
