use cargo_util_schemas::manifest::TomlManifest;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use log::warn;

#[derive(Debug, PartialEq)]
pub enum WorkspaceMemberStatus {
    Added(PathBuf),
    NoWorkspaceFound,
}

/// Add the given project to the workspace members list.
/// If there is no workspace project in the parent directories of the given path, do nothing.
pub fn add_to_workspace(member_path: &Path) -> Result<WorkspaceMemberStatus> {
    let Some(mut workspace) = Workspace::try_new(member_path)? else {
        return Ok(WorkspaceMemberStatus::NoWorkspaceFound);
    };
    let member = WorkspaceMember::try_new(member_path)?;
    workspace.add_member(member)?;
    workspace.save()?;

    Ok(WorkspaceMemberStatus::Added(workspace.cargo_toml_path))
}

struct Workspace {
    manifest: TomlManifest,
    cargo_toml_path: PathBuf,
}

impl Workspace {
    /// Try to find a workspace project in the parent directories of the given path.
    ///
    /// Returns `None` if no workspace project is found.
    pub fn try_new(member_path: &Path) -> Result<Option<Self>> {
        if let Some(parent) = member_path.parent() {
            let cargo_toml_path = parent.join("Cargo.toml");
            if cargo_toml_path.exists() {
                let content = fs::read_to_string(&cargo_toml_path)?;
                let manifest: TomlManifest = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;
                if manifest.workspace.is_some()
                    && manifest.workspace.as_ref().unwrap().members.is_some()
                {
                    return Ok(Some(Self {
                        manifest,
                        cargo_toml_path,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Add a new member to the workspace, if it is not already a member.
    /// The member list will be sorted alphabetically.
    pub fn add_member(&mut self, member: WorkspaceMember) -> Result<()> {
        let Some(workspace) = self.manifest.workspace.as_mut() else {
            bail!(
                "There is no workspace project at {}",
                self.cargo_toml_path.display()
            );
        };

        let Some(members) = workspace.members.as_mut() else {
            bail!("There are no workspace members yet defined.");
        };

        if members.contains(&member.name) {
            warn!(
                "Project `{}` is already a member of the workspace",
                member.name
            );
            return Ok(());
        }

        members.push(member.name.clone());
        members.sort();

        Ok(())
    }

    /// Save the updated manifest to disk.
    pub fn save(&self) -> Result<()> {
        let new_manifest = toml::to_string_pretty(&self.manifest)?;
        let cargo_toml_path = &self.cargo_toml_path;
        fs::write(cargo_toml_path, new_manifest)
            .with_context(|| format!("Failed to write {}", cargo_toml_path.display()))?;

        Ok(())
    }
}

struct WorkspaceMember {
    name: String,
}

impl WorkspaceMember {
    pub fn try_new(member_path: &Path) -> Result<Self> {
        let cargo_toml_path = member_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;
        let manifest: TomlManifest = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;

        let name = manifest.package().unwrap().name.as_ref().to_string();

        Ok(Self { name })
    }
}
