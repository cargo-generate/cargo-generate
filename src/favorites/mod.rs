//! `list_favorites` output. Two implementations coexist behind the `ui-next`
//! feature.

#[cfg(not(feature = "ui-next"))]
mod classic;
#[cfg(feature = "ui-next")]
mod next;

#[cfg(not(feature = "ui-next"))]
#[allow(deprecated)]
pub use classic::*;

#[cfg(feature = "ui-next")]
pub use next::*;
