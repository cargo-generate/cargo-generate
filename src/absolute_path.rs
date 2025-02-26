use std::{
    env,
    path::{Component, Path, PathBuf},
};

/// This trait extends `Path` and `PathBuf` to provide a method to convert a relative path to an absolute path.
pub trait AbsolutePathExt {
    /// Converts a relative path to an absolute path.
    fn as_absolute(&self) -> std::result::Result<PathBuf, std::io::Error>;

    /// Converts a relative path to an absolute path and checks if it is sandboxed within a given directory.
    fn as_sandboxed_absolute(&self, sandbox: &Path)
        -> std::result::Result<PathBuf, std::io::Error>;
}

impl AbsolutePathExt for PathBuf {
    fn as_absolute(&self) -> std::result::Result<PathBuf, std::io::Error> {
        if self.is_absolute() {
            return Ok(self.clone());
        }

        let cwd = env::current_dir()?;
        Ok(canonicalize_path(&cwd.join(self)))
    }

    fn as_sandboxed_absolute(
        &self,
        sandbox: &Path,
    ) -> std::result::Result<PathBuf, std::io::Error> {
        let sandbox = canonicalize_path(sandbox);

        let absolute_path = if self.is_relative() {
            canonicalize_path(&sandbox.join(self))
        } else {
            canonicalize_path(self)
        };

        if !absolute_path.starts_with(sandbox) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Path cannot escape the sandbox",
            ));
        }
        Ok(absolute_path)
    }
}

impl AbsolutePathExt for std::path::Path {
    fn as_absolute(&self) -> std::result::Result<PathBuf, std::io::Error> {
        PathBuf::from(self).as_absolute()
    }

    fn as_sandboxed_absolute(
        &self,
        sandbox: &Path,
    ) -> std::result::Result<PathBuf, std::io::Error> {
        PathBuf::from(self).as_sandboxed_absolute(sandbox)
    }
}

/// Canonicalizes a path without requiring the files and folders to exist.
fn canonicalize_path(path: &Path) -> PathBuf {
    let components = path.components().peekable();
    let mut result = PathBuf::new();

    for component in components {
        match component {
            Component::RootDir => {
                result.push(component);
            }
            Component::CurDir => {
                // Skip current directory components
            }
            Component::ParentDir => {
                // Pop the last component if it's not a root directory
                if result.components().last() != Some(Component::RootDir) {
                    result.pop();
                }
            }
            Component::Normal(_) => {
                result.push(component);
            }
            Component::Prefix(prefix) => match prefix.kind() {
                std::path::Prefix::Disk(_) => {
                    result.push(prefix.as_os_str());
                }
                _ => {
                    // Skip other types of prefixes
                }
            },
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_non_existing_path() {
        #[cfg(target_family = "unix")]
        let relative_path = PathBuf::from("/non-existing-path");
        #[cfg(target_family = "windows")]
        let relative_path = PathBuf::from("D:\\non-existing-path");
        let absolute_path = relative_path.as_absolute().unwrap();
        assert!(absolute_path.is_absolute());
        assert_eq!(absolute_path, relative_path);
    }

    #[test]
    fn test_relative_path_to_absolute() {
        let cwd = env::current_dir().expect("Failed to get current working directory");

        // Test with a relative path
        let relative_path = PathBuf::from("src/main.rs");
        let absolute_path = relative_path.as_absolute().unwrap();
        assert!(absolute_path.is_absolute());
        assert!(absolute_path.starts_with(&cwd));
    }

    #[test]
    fn test_absolute_path_remains_unchanged() {
        let cwd = env::current_dir().expect("Failed to get current working directory");

        // Test with an absolute path
        let absolute_path_input = cwd.join("src/main.rs");
        let absolute_path_output = absolute_path_input.as_absolute().unwrap();
        assert_eq!(absolute_path_input, absolute_path_output);
    }

    #[test]
    fn test_complex_relative_path_to_absolute() {
        let cwd = env::current_dir().expect("Failed to get current working directory");

        // Test with a path containing `.` and `..`
        let complex_relative_path = PathBuf::from("src/./../src/main.rs");
        let complex_absolute_path = complex_relative_path.as_absolute().unwrap();
        assert!(complex_absolute_path.is_absolute());
        assert!(complex_absolute_path.starts_with(&cwd));
        assert!(complex_absolute_path.ends_with("src/main.rs"));
    }

    #[test]
    fn test_empty_path() {
        let cwd = env::current_dir().expect("Failed to get current working directory");

        // Test with an empty path
        let empty_path = PathBuf::new();
        let absolute_path = empty_path.as_absolute().unwrap();
        assert_eq!(absolute_path, cwd);
    }

    #[test]
    fn test_root_path() {
        // Test with the root path
        #[cfg(target_family = "unix")]
        let root_path = PathBuf::from("/");
        #[cfg(target_family = "windows")]
        let root_path = PathBuf::from("D:\\");

        let absolute_path = root_path.as_absolute().unwrap();
        assert_eq!(absolute_path, root_path);
    }

    #[test]
    fn test_dot_path() {
        let cwd = env::current_dir().expect("Failed to get current working directory");

        // Test with a single dot path
        let dot_path = PathBuf::from(".");
        let absolute_path = dot_path.as_absolute().unwrap();
        assert_eq!(absolute_path, cwd);
    }

    #[test]
    fn test_dot_dot_path() {
        let cwd = env::current_dir().expect("Failed to get current working directory");
        let parent_dir = cwd.parent().expect("Failed to get parent directory");

        // Test with a double dot path
        let dot_dot_path = PathBuf::from("..");
        let absolute_path = dot_dot_path.as_absolute().unwrap();
        assert_eq!(absolute_path, parent_dir);
    }

    #[test]
    fn test_multiple_consecutive_slashes() {
        let cwd = env::current_dir().expect("Failed to get current working directory");

        // Test with multiple consecutive slashes
        let multiple_slashes_path = PathBuf::from("src//main.rs");
        let absolute_path = multiple_slashes_path.as_absolute().unwrap();
        assert!(absolute_path.is_absolute());
        assert!(absolute_path.starts_with(&cwd));
        assert!(absolute_path.ends_with("src/main.rs"));
    }

    #[test]
    fn test_path_within_sandbox() {
        #[cfg(target_family = "unix")]
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(target_family = "windows")]
        let sandbox = PathBuf::from("D:\\sandbox");
        let relative_path = PathBuf::from("file.txt");
        let absolute_path = relative_path.as_sandboxed_absolute(&sandbox).unwrap();
        assert!(absolute_path.is_absolute());
        assert!(absolute_path.starts_with(&sandbox));
    }

    #[test]
    #[should_panic(expected = "Path cannot escape the sandbox")]
    fn test_path_outside_sandbox() {
        #[cfg(target_family = "unix")]
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(target_family = "windows")]
        let sandbox = PathBuf::from("D:\\sandbox");        let relative_path = PathBuf::from("/outside/file.txt");
        relative_path.as_sandboxed_absolute(&sandbox).unwrap();
    }

    #[test]
    #[should_panic(expected = "Path cannot escape the sandbox")]
    fn test_path_with_dot_dot_to_escape_sandbox() {
        #[cfg(target_family = "unix")]
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(target_family = "windows")]
        let sandbox = PathBuf::from("D:\\sandbox");        let relative_path = PathBuf::from("../file.txt");
        relative_path.as_sandboxed_absolute(&sandbox).unwrap();
    }

    #[test]
    fn test_empty_path_in_sandbox() {
        #[cfg(target_family = "unix")]
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(target_family = "windows")]
        let sandbox = PathBuf::from("D:\\sandbox");
        let empty_path = PathBuf::new();
        let absolute_path = empty_path.as_sandboxed_absolute(&sandbox).unwrap();
        assert_eq!(absolute_path, sandbox);
    }

    #[test]
    fn test_root_path_in_sandbox() {
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(unix)]
        let root_path = PathBuf::from("/");
        #[cfg(windows)]
        let root_path = PathBuf::from("D:\\");

        let result = root_path.as_sandboxed_absolute(&sandbox);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_consecutive_slashes_in_sandbox() {
        #[cfg(target_family = "unix")]
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(target_family = "unix")]
        let multiple_slashes_path = PathBuf::from("src//main.rs");

        #[cfg(target_family = "windows")]
        let sandbox = PathBuf::from("D:\\sandbox");
        #[cfg(target_family = "windows")]
        let multiple_slashes_path = PathBuf::from("src\\\\main.rs");

        let absolute_path = multiple_slashes_path
            .as_sandboxed_absolute(&sandbox)
            .unwrap();
        assert!(absolute_path.is_absolute());
        assert!(absolute_path.starts_with(&sandbox));
        #[cfg(target_family = "unix")]
        assert!(absolute_path.ends_with("src/main.rs"));
        #[cfg(target_family = "windows")]
        assert!(absolute_path.ends_with("src\\main.rs"));
    }

    #[test]
    #[should_panic(expected = "Path cannot escape the sandbox")]
    fn test_path_with_multiple_dot_dot_to_escape_sandbox() {
        let sandbox = PathBuf::from("/sandbox");
        let relative_path = PathBuf::from("///../../../....//");
        relative_path.as_sandboxed_absolute(&sandbox).unwrap();
    }

    #[test]
    #[should_panic(expected = "Path cannot escape the sandbox")]
    fn test_path_with_nested_dot_dot_to_escape_sandbox() {
        let sandbox = PathBuf::from("/sandbox");
        let relative_path = PathBuf::from("nested/../../../../file.txt");
        relative_path.as_sandboxed_absolute(&sandbox).unwrap();
    }

    #[test]
    #[should_panic(expected = "Path cannot escape the sandbox")]
    fn test_path_with_mixed_dots_and_slashes() {
        let sandbox = PathBuf::from("/sandbox");
        let relative_path = PathBuf::from("./././.././../file.txt");
        relative_path.as_sandboxed_absolute(&sandbox).unwrap();
    }

    #[test]
    fn test_path_with_trailing_slashes() {
        #[cfg(target_family = "unix")]
        let sandbox = PathBuf::from("/sandbox");
        #[cfg(target_family = "unix")]
        let relative_path = PathBuf::from("file.txt///");

        #[cfg(target_family = "windows")]
        let sandbox = PathBuf::from("D:\\sandbox");
        #[cfg(target_family = "windows")]
        let relative_path = PathBuf::from("file.txt\\\\\\");

        let absolute_path = relative_path.as_sandboxed_absolute(&sandbox).unwrap();
        assert!(absolute_path.is_absolute());
        assert!(absolute_path.starts_with(&sandbox));
    }

    #[test]
    #[should_panic(expected = "Path cannot escape the sandbox")]
    fn test_path_with_leading_slashes() {
        let sandbox = PathBuf::from("/sandbox");
        let some_malicious_path = PathBuf::from("///file.txt");
        some_malicious_path.as_sandboxed_absolute(&sandbox).unwrap();
    }
}
