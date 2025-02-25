use console::style;
use path_absolutize::Absolutize;
use rhai::{Array, Dynamic, Module};
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
        let dir = dir.clone();

        move |file: &str, content: Array| -> HookResult<()> {
            let file = to_absolute_path(&dir, file)?;
            let mut file = std::fs::File::create(file).map_err(|e| e.to_string())?;
            for v in content.iter() {
                writeln!(file, "{v}").map_err(|e| e.to_string())?;
            }

            Ok(())
        }
    });

    // listdir(path);
    module.set_native_fn("listdir", {
        let dir = dir.clone();
        move |path: &str| -> HookResult<Array> { listdir(&dir, path) }
    });

    // listdir(path = ".");
    module.set_native_fn("listdir", {
        let dir = dir.clone();
        move || -> HookResult<Array> { listdir(&dir, ".") }
    });

    module
}

fn listdir(base_dir: &Path, path: &str) -> HookResult<Array> {
    let entries = std::fs::read_dir(to_absolute_path(base_dir, path)?)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
        .map(Dynamic::from)
        .collect::<Array>();

    Ok(entries)
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        hooks::{create_rhai_engine, RhaiHooksContext},
        template::LiquidObjectResource,
    };
    use rhai::Array;
    use tempfile::TempDir;

    #[test]
    fn test_file_module() {
        let tmp_dir = TempDir::new().unwrap();
        let mut file1 = std::fs::File::create(tmp_dir.path().join("file1")).unwrap();
        file1.write_all(b"test1").unwrap();
        let mut file2 = std::fs::File::create(tmp_dir.path().join(".dotfile")).unwrap();
        file2.write_all(b"test1").unwrap();
        let context = RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: true,
            silent: true,
        };
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let files = engine.eval::<Array>("file::listdir()").unwrap();
        assert_eq!(files.len(), 2);
        let file = files.first().unwrap().clone();
        assert!(file.into_string().unwrap().as_str().ends_with(".dotfile"));
        let file = files[1].clone();
        assert!(file.into_string().unwrap().as_str().ends_with("file1"));

        // cover the other listdir function with one path argument
        let files = engine.eval::<Array>(r#"file::listdir(".")"#).unwrap();
        assert_eq!(files.len(), 2);
        let file = files.first().unwrap().clone();
        assert!(file.into_string().unwrap().as_str().ends_with(".dotfile"));
        let file = files[1].clone();
        assert!(file.into_string().unwrap().as_str().ends_with("file1"));
    }

    #[test]
    #[should_panic(expected = "Path must be inside template dir:")]
    fn test_listdir_error_case() {
        let tmp_dir = TempDir::new().unwrap();
        let context = RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: true,
            silent: true,
        };
        let engine = create_rhai_engine(&context);
        engine.eval::<Array>(r#"file::listdir("/tmp")"#).unwrap();
    }
}
