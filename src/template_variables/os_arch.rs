use std::env;

pub type OsArch = String;

pub fn get_os_arch() -> OsArch {
    format!("{}-{}", env::consts::OS, env::consts::ARCH)
}
