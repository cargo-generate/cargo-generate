use std::path::PathBuf;

use rhai::Module;

pub struct Environment {
    /// The temp directory where the template repository is pre-processed
    pub tmp_dir: PathBuf,
    /// The destination directory where the template repository is copied to
    pub destination_dir: PathBuf,
}

impl Environment {
    pub fn tmp(&self) -> &str {
        self.tmp_dir.to_str().unwrap()
    }

    pub fn destination(&self) -> &str {
        self.destination_dir.to_str().unwrap()
    }
}

/// Creates the system module, containing the `command` function,
/// which allows you to run system command.
pub fn create_module(env: Environment) -> Module {
    let mut module = Module::new();

    module.set_var("tmp", env.tmp().to_string());
    module.set_var("destination", env.destination().to_string());

    module
}
