use crate::git::utils::home;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use std::path::{Path, PathBuf};
use thiserror::Error;

//FIXME: this could be io::Error
#[derive(Debug, Clone, Error)]
pub enum IdentityPathErr {
    #[error("identity file do not exist")]
    FileNoExist,
    #[error("path is not a file")]
    NotFile,
}

pub struct IdentityPath(PathBuf);

impl TryFrom<PathBuf> for IdentityPath {
    type Error = IdentityPathErr;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if path.exists() {
            if path.is_file() {
                Ok(Self(path))
            } else {
                Err(IdentityPathErr::NotFile)
            }
        } else {
            Err(IdentityPathErr::FileNoExist)
        }
    }
}

impl Display for IdentityPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pretty_path(self.as_ref()).unwrap())
    }
}

impl AsRef<Path> for IdentityPath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

/// prevents from long stupid paths, and replace the home path by the literal `$HOME`
pub fn pretty_path(a: &Path) -> Result<String> {
    #[cfg(not(windows))]
    let home_var = "$HOME";
    #[cfg(windows)]
    let home_var = "%userprofile%";
    Ok(a.display()
        .to_string()
        .replace(&home()?.display().to_string(), home_var))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pretty_path() {
        let p = pretty_path(home().unwrap().as_path().join(".cargo").as_path()).unwrap();
        #[cfg(unix)]
        assert_eq!(p, "$HOME/.cargo");
        #[cfg(windows)]
        assert_eq!(p, "%userprofile%\\.cargo");
    }
}
