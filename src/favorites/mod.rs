//! `list_favorites` output. Two implementations coexist behind the `ui-next`
//! feature.

#[cfg(not(feature = "ui-next"))]
mod ui_classic;
#[cfg(feature = "ui-next")]
mod ui_next;

#[cfg(not(feature = "ui-next"))]
#[allow(deprecated)]
pub use ui_classic::*;

#[cfg(feature = "ui-next")]
pub use ui_next::*;
