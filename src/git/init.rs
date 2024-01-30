use anyhow::Result;
use std::path::Path;

/// Init project_dir with fresh repository on branch
///
/// Arguments:
/// - `force` - enforce a fresh git init
pub fn init(project_dir: &Path, branch: Option<&str>, force: bool) -> Result<git2::Repository> {
    git2::Repository::discover(project_dir).map_or_else(
        |_| just_init(project_dir, branch),
        |repo| {
            if force {
                git2::Repository::open(project_dir).or_else(|_| just_init(project_dir, branch))
            } else {
                Ok(repo)
            }
        },
    )
}

fn just_init(project_dir: &Path, branch: Option<&str>) -> Result<git2::Repository> {
    let mut opts = git2::RepositoryInitOptions::new();
    opts.bare(false);
    if let Some(branch) = branch {
        opts.initial_head(branch);
    }
    Ok(git2::Repository::init_opts(project_dir, &opts)?)
}
