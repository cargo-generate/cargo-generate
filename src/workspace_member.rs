use anyhow::{anyhow, Context, Result};
use cargo_util_schemas::manifest::TomlManifest;
use glob::Pattern;
use log::warn;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub enum WorkspaceMemberStatus {
    /// The new project was appended to the workspace's `members` list.
    Added(PathBuf),
    /// A workspace root was found, but the new project's directory matches an
    /// entry in `workspace.exclude`; the manifest was left untouched.
    Excluded(PathBuf),
    /// A workspace root was found and the new project is already implicitly
    /// covered by an existing glob entry in `members`; no manifest change was
    /// required.
    AlreadyCoveredByGlob(PathBuf),
    /// No workspace root was found while walking up the parent directories.
    NoWorkspaceFound,
}

/// Add the given project to the workspace's `members` list.
///
/// Walks up from `member_path` looking for the first `Cargo.toml` that has a
/// `[workspace]` table, mirroring `cargo new` (see `find_root_manifest_for_wd`
/// and `update_manifest_with_new_member` in `rust-lang/cargo`).
pub fn add_to_workspace(member_path: &Path) -> Result<WorkspaceMemberStatus> {
    let Some(mut workspace) = Workspace::find_root(member_path)? else {
        return Ok(WorkspaceMemberStatus::NoWorkspaceFound);
    };
    let relative_member_path = workspace.relative_path_for(member_path)?;

    if workspace.is_excluded(&relative_member_path) {
        warn!(
            "Project `{}` matches `workspace.exclude` in {}; skipping workspace membership.",
            relative_member_path,
            workspace.cargo_toml_path.display()
        );
        return Ok(WorkspaceMemberStatus::Excluded(workspace.cargo_toml_path));
    }

    if workspace.is_covered_by_existing_member(&relative_member_path) {
        return Ok(WorkspaceMemberStatus::AlreadyCoveredByGlob(
            workspace.cargo_toml_path,
        ));
    }

    workspace.append_member(relative_member_path)?;
    workspace.save()?;
    Ok(WorkspaceMemberStatus::Added(workspace.cargo_toml_path))
}

struct Workspace {
    manifest: TomlManifest,
    cargo_toml_path: PathBuf,
    root_dir: PathBuf,
}

impl Workspace {
    fn find_root(member_path: &Path) -> Result<Option<Self>> {
        for ancestor in member_path.ancestors().skip(1) {
            let cargo_toml_path = ancestor.join("Cargo.toml");
            if !cargo_toml_path.exists() {
                continue;
            }
            let content = fs::read_to_string(&cargo_toml_path)?;
            let manifest: TomlManifest = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;
            if manifest.workspace.is_some() {
                return Ok(Some(Self {
                    manifest,
                    cargo_toml_path,
                    root_dir: ancestor.to_path_buf(),
                }));
            }
        }
        Ok(None)
    }

    /// Path from the workspace root to `member_path`, joined with `/` so the
    /// value written into `Cargo.toml` matches cargo's convention on every
    /// platform (see `get_display_path` in `rust-lang/cargo`).
    fn relative_path_for(&self, member_path: &Path) -> Result<String> {
        let rel = member_path.strip_prefix(&self.root_dir).with_context(|| {
            format!(
                "Project path {} is not inside workspace root {}",
                member_path.display(),
                self.root_dir.display()
            )
        })?;
        Ok(rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"))
    }

    fn is_excluded(&self, relative_member_path: &str) -> bool {
        self.manifest
            .workspace
            .as_ref()
            .and_then(|ws| ws.exclude.as_ref())
            .is_some_and(|exclude| exclude.iter().any(|e| e == relative_member_path))
    }

    fn is_covered_by_existing_member(&self, relative_member_path: &str) -> bool {
        let Some(members) = self
            .manifest
            .workspace
            .as_ref()
            .and_then(|ws| ws.members.as_ref())
        else {
            return false;
        };
        members.iter().any(|entry| {
            if entry == relative_member_path {
                return true;
            }
            Pattern::new(entry)
                .map(|p| p.matches(relative_member_path))
                .unwrap_or(false)
        })
    }

    fn append_member(&mut self, relative_member_path: String) -> Result<()> {
        let workspace = self.manifest.workspace.as_mut().ok_or_else(|| {
            anyhow!(
                "There is no workspace project at {}",
                self.cargo_toml_path.display()
            )
        })?;
        let members = workspace.members.get_or_insert_with(Vec::new);
        let was_sorted = members.windows(2).all(|w| w[0] <= w[1]);
        members.push(relative_member_path);
        if was_sorted {
            members.sort();
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let new_manifest = toml::to_string_pretty(&self.manifest)?;
        fs::write(&self.cargo_toml_path, new_manifest)
            .with_context(|| format!("Failed to write {}", self.cargo_toml_path.display()))?;
        Ok(())
    }
}
