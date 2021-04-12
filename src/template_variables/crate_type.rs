use crate::Args;

pub(crate) type CrateType = cargo::core::compiler::CrateType;

impl From<&Args> for CrateType {
    fn from(a: &Args) -> CrateType {
        if a.lib {
            CrateType::Lib
        } else {
            CrateType::Bin
        }
    }
}
