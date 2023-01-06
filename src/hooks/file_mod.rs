use console::style;
use path_absolutize::Absolutize;
use rhai::{Array, Module};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::HookResult;

pub fn create_module(dir: &Path) -> Module {
    let dir = dir.to_owned();
    let mut module = Module::new();

    module.set_native_fn("exists", {
        let dir = dir.clone();

        move |path: &str| -> HookResult<bool> {
            let path = to_absolute_path(&dir, path)?;
            Ok(path.exists())
        }
    });

    module.set_native_fn("rename", {
        let dir = dir.clone();

        move |from: &str, to: &str| -> HookResult<()> {
            let from = to_absolute_path(&dir, from)?;
            let to = to_absolute_path(&dir, to)?;
            std::fs::rename(from, to).map_err(|e| e.to_string())?;
            Ok(())
        }
    });

    module.set_native_fn("delete", {
        let dir = dir.clone();

        move |file: &str| -> HookResult<()> {
            let path = to_absolute_path(&dir, file)?;
            if path.exists() {
                if path.is_file() {
                    std::fs::remove_file(path).map_err(|e| e.to_string())?;
                } else {
                    std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
                }
            }
            Ok(())
        }
    });

    module.set_native_fn("write", {
        let dir = dir.clone();

        move |file: &str, content: &str| -> HookResult<()> {
            let file = to_absolute_path(&dir, file)?;
            std::fs::write(file, content).map_err(|e| e.to_string())?;
            Ok(())
        }
    });

    module.set_native_fn("write", {
        move |file: &str, content: Array| -> HookResult<()> {
            let file = to_absolute_path(&dir, file)?;
            let mut file = std::fs::File::create(file).map_err(|e| e.to_string())?;
            for v in content.iter() {
                writeln!(file, "{v}").map_err(|e| e.to_string())?;
            }

            Ok(())
        }
    });

    module
}

fn to_absolute_path(base_dir: &Path, relative_path: &str) -> HookResult<PathBuf> {
    let joined = base_dir.join(relative_path);
    Ok(joined
        .absolutize_virtually(base_dir)
        .map_err(|_| invalid_path(relative_path))?
        .into_owned())
}

fn invalid_path(path: &str) -> String {
    format!(
        "{} {}",
        style("Path must be inside template dir:").bold().red(),
        style(path).yellow(),
    )
}
