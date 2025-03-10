use std::path::PathBuf;

use crate::template::LiquidObjectResource;

#[derive(Debug)]
pub struct RhaiHooksContext {
    pub liquid_object: LiquidObjectResource,
    pub allow_commands: bool,
    pub silent: bool,
    pub working_directory: PathBuf,
    pub destination_directory: PathBuf,
}
