use crate::Args;

pub type CrateType = cargo::core::compiler::CrateType;

impl From<&Args> for CrateType {
    fn from(a: &Args) -> Self {
        if a.lib {
            Self::Lib
        } else {
            Self::Bin
        }
    }
}
