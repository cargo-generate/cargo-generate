use ignore::gitignore::GitignoreBuilder;
use ignore::Match;
use std::path::Path;
use std::iter::Iterator;
use crate::config::TemplateConfig;
use walkdir::{DirEntry, WalkDir, Result as WDResult};

pub fn create_matcher(template_config: &TemplateConfig, project_dir: &Path) -> Result<impl Iterator<Item = WDResult<DirEntry>>, failure::Error> {
    let mut exclude_builder = GitignoreBuilder::new(project_dir);
    for rule in template_config.include.unwrap_or_else(Vec::new) {
        exclude_builder.add_line(None, &rule)?;
    }
    let exclude_matcher = exclude_builder.build()?;

    let mut include_builder = GitignoreBuilder::new(project_dir);
    for rule in template_config.exclude.unwrap_or_else(Vec::new) {
        include_builder.add_line(None, &rule)?;
    }
    let include_matcher = include_builder.build()?;

    let should_include = |relative_path: &Path| -> Result<bool, failure::Error> {
        // "Include" and "exclude" options are mutually exclusive.
        // if no include is made, we will default to ignore_exclude
        // which if there is no options, matches everything
        if template_config.include.is_none() {
            match exclude_matcher
                .matched_path_or_any_parents(relative_path, /* is_dir */ false)
            {
                Match::None => Ok(true),
                Match::Ignore(_) => Ok(false),
                Match::Whitelist(_) => Ok(true),
            }
        } else {
            match include_matcher
                .matched_path_or_any_parents(relative_path, /* is_dir */ false)
            {
                Match::None => Ok(false),
                Match::Ignore(_) => Ok(true),
                Match::Whitelist(_) => Ok(false),
            }
        }
    };

    fn is_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
    }

    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .to_str()
            .map(|s| s.contains(".git"))
            .unwrap_or(false)
    }

    Ok(WalkDir::new(project_dir)
        .into_iter()
        .filter_entry(|e| {
            let relative_path = e.path().strip_prefix(project_dir).expect("strip project dir before matching");
            !is_dir(e) && !is_git_metadata(e) && should_include(relative_path).unwrap()
        }).into_iter())
}

