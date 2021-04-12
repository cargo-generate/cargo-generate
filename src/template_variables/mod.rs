mod authors;
mod crate_type;
mod os_arch;
mod project_name;

pub(crate) use authors::{get_authors, Authors};
pub(crate) use crate_type::CrateType;
pub(crate) use os_arch::get_os_arch;
pub(crate) use project_name::ProjectName;
