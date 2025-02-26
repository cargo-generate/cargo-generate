use std::path::PathBuf;

use rhai::Module;

pub struct Environment {
    /// The temp directory where the template repository is pre-processed
    pub working_directory: PathBuf,
    /// The destination directory where the template repository is copied to
    pub destination_directory: PathBuf,
}

/// Creates the system module, containing the `command` function,
/// which allows you to run system command.
pub fn create_module(env: Environment) -> Module {
    let mut module = Module::new();

    module.set_var(
        "working_directory",
        env.working_directory.to_string_lossy().to_string(),
    );
    module.set_var(
        "destination_directory",
        env.destination_directory.to_string_lossy().to_string(),
    );

    module
}

#[cfg(test)]
mod tests {
    use crate::{
        hooks::{create_rhai_engine, RhaiHooksContext},
        template::LiquidObjectResource,
    };
    use tempfile::TempDir;

    #[test]
    fn test_env_module() {
        let tmp_dir = TempDir::new().unwrap();
        let context = RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: true,
            silent: true,
        };
        let engine = create_rhai_engine(&context);

        let working_directory = engine.eval::<String>("env::working_directory").unwrap();
        assert_eq!(
            working_directory,
            context.working_directory.to_string_lossy()
        );
        let destination_directory = engine.eval::<String>("env::destination_directory").unwrap();
        assert_eq!(
            destination_directory,
            context.destination_directory.to_string_lossy()
        );
    }
}
