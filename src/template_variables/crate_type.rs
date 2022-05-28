use std::fmt;

use crate::Args;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CrateType {
    Bin,
    Lib,
}

impl fmt::Display for CrateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bin => write!(f, "bin"),
            Self::Lib => write!(f, "lib"),
        }
    }
}

impl From<&Args> for CrateType {
    fn from(a: &Args) -> Self {
        if a.lib {
            Self::Lib
        } else {
            Self::Bin
        }
    }
}
