use cargo_util_schemas::manifest::TomlManifest;
use console::style;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use log::{info, warn};

use crate::emoji;

pub fn add_to_workspace(member_path: &Path) -> Result<()> {
    let Some(mut workspace) = Workspace::new(member_path)? else {
        info!("No workspace project found.");
        return Ok(());
    };
    let member = WorkspaceMember::new(member_path)?;
    workspace.add_member(member)?;
    workspace.save()?;

    Ok(())
}

struct Workspace {
    manifest: TomlManifest,
    cargo_toml_path: PathBuf,
}

impl Workspace {
    pub fn new(member_path: &Path) -> Result<Option<Self>> {
        let current_path = member_path;

        if let Some(parent) = current_path.parent() {
            let cargo_toml_path = parent.join("Cargo.toml");
            if cargo_toml_path.exists() {
                let content = fs::read_to_string(&cargo_toml_path)?;
                let manifest: TomlManifest = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;
                if manifest.workspace.is_some() {
                    return Ok(Some(Self {
                        manifest,
                        cargo_toml_path,
                    }));
                }
            }
        }

        Ok(None)
    }

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

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let new_manifest = toml::to_string_pretty(&self.manifest)?;
        let cargo_toml_path = &self.cargo_toml_path;
        fs::write(cargo_toml_path, new_manifest)
            .with_context(|| format!("Failed to write {}", cargo_toml_path.display()))?;

        info!(
            "{} {} `{}`",
            emoji::WRENCH,
            style("Project added as member to workspace").bold(),
            style(cargo_toml_path.display()).bold().yellow(),
        );

        Ok(())
    }
}

struct WorkspaceMember {
    name: String,
}

impl WorkspaceMember {
    pub fn new(member_path: &Path) -> Result<Self> {
        let cargo_toml_path = member_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;
        let manifest: TomlManifest = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;

        let name = manifest.package().unwrap().name.as_ref().to_string();

        Ok(Self { name })
    }
}
