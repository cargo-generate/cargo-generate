use crate::absolute_path::AbsolutePathExt;
use console::style;
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
            let path = to_sandboxed_absolute_path(&dir, path)?;
            Ok(path.exists())
        }
    });

    module.set_native_fn("rename", {
        let dir = dir.clone();

        move |from: &str, to: &str| -> HookResult<()> {
            let from = to_sandboxed_absolute_path(&dir, from)?;
            let to = to_sandboxed_absolute_path(&dir, to)?;
            std::fs::rename(from, to).map_err(|e| e.to_string())?;
            Ok(())
        }
    });

    module.set_native_fn("delete", {
        let dir = dir.clone();

        move |file: &str| -> HookResult<()> {
            let path = to_sandboxed_absolute_path(&dir, file)?;
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
            let file = to_sandboxed_absolute_path(&dir, file)?;
            std::fs::write(file, content).map_err(|e| e.to_string())?;
            Ok(())
        }
    });

    module.set_native_fn("write", {
        let dir = dir.clone();

        move |file: &str, content: Array| -> HookResult<()> {
            let file = to_sandboxed_absolute_path(&dir, file)?;
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
    let entries = std::fs::read_dir(to_sandboxed_absolute_path(base_dir, path)?)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
        .map(Dynamic::from)
        .collect::<Array>();

    Ok(entries)
}

fn to_sandboxed_absolute_path(sandbox_dir: &Path, any_path: &str) -> HookResult<PathBuf> {
    Ok(PathBuf::from(any_path)
        .as_sandboxed_absolute(sandbox_dir)
        .map_err(|_| invalid_path(any_path))?)
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
    fn test_to_sandboxed_absolute_path() {
        let tmp_dir = prepare_file_system();

        let absolute_path =
            super::to_sandboxed_absolute_path(tmp_dir.path(), "Cargo.toml").unwrap();
        assert_eq!(absolute_path, tmp_dir.path().join("Cargo.toml"));

        let path = std::path::Path::new(".");
        let absolute_path =
            super::to_sandboxed_absolute_path(tmp_dir.path(), path.to_str().unwrap()).unwrap();
        assert_eq!(absolute_path, tmp_dir.path());
    }

    #[test]
    fn test_file_module() {
        let tmp_dir = prepare_file_system();
        let context = prepare_context(&tmp_dir);
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let files = engine.eval::<Array>("file::listdir()").unwrap();
        assert_eq!(files.len(), 2);
        for file in files.iter() {
            let file = file.clone().into_string().unwrap();
            assert!(
                file.ends_with(".dotfile") || file.ends_with("file1"),
                "file: {file} did not end with .dotfile nor file1"
            );
        }
    }

    #[test]
    fn test_listdir_with_parameter() {
        let tmp_dir = prepare_file_system();
        let context = prepare_context(&tmp_dir);
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        // cover the other listdir function with one path argument
        let files = engine.eval::<Array>(r#"file::listdir(".")"#).unwrap();
        assert_eq!(files.len(), 2);
        for file in files.iter() {
            let file = file.clone().into_string().unwrap();
            assert!(
                file.ends_with(".dotfile") || file.ends_with("file1"),
                "file: {file} did not end with .dotfile nor file1"
            );
        }
    }

    #[test]
    #[cfg(target_family = "unix")]
    #[should_panic(expected = "Path must be inside template dir:")]
    fn test_listdir_does_not_escape_sandboxed_directory() {
        let tmp_dir = prepare_file_system();
        let context = prepare_context(&tmp_dir);
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let path_argument = "../../../../etc";
        engine
            .eval::<Array>(format!("file::listdir(\"{path_argument}\")").as_str())
            .unwrap();
    }

    #[test]
    #[cfg(target_family = "unix")]
    #[should_panic(expected = "Path must be inside template dir:")]
    fn test_listdir_does_not_escape_sandboxed_directory_2() {
        let tmp_dir = prepare_file_system();
        let context = prepare_context(&tmp_dir);
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let path_argument = "////../../../../etc";
        engine
            .eval::<Array>(format!("file::listdir(\"{path_argument}\")").as_str())
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Path must be inside template dir:")]
    fn test_listdir_error_case() {
        let tmp_dir = TempDir::new().unwrap();
        let context = prepare_context(&tmp_dir);
        let engine = create_rhai_engine(&context);
        engine.eval::<Array>(r#"file::listdir("/tmp")"#).unwrap();
    }

    fn prepare_file_system() -> TempDir {
        let tmp_dir = TempDir::new().unwrap();
        let mut file1 = std::fs::File::create(tmp_dir.path().join("file1")).unwrap();
        file1.write_all(b"test1").unwrap();
        let mut file2 = std::fs::File::create(tmp_dir.path().join(".dotfile")).unwrap();
        file2.write_all(b"test1").unwrap();
        tmp_dir
    }

    fn prepare_context(tmp_dir: &TempDir) -> RhaiHooksContext {
        RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: true,
            silent: true,
        }
    }
}
