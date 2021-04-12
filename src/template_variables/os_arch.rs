use std::env;

pub(crate) type OsArch = String;

pub(crate) fn get_os_arch() -> OsArch {
    format!("{}-{}", env::consts::OS, env::consts::ARCH)
}
