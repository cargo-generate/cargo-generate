use crate::git::utils::{canonicalize_path, home};
use std::fmt::{Display, Formatter};

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub struct IdentityPath(PathBuf);

impl TryFrom<PathBuf> for IdentityPath {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let path_can = canonicalize_path(&path)?;

        if path_can.exists() {
            Ok(Self(path_can))
        } else {
            bail!("Invalid ssh identity path: {}", path.as_path().display())
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
