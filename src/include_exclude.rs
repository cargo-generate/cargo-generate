use crate::config::TemplateConfig;
use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

#[derive(Default)]
pub(crate) struct Matcher(Option<MatcherKind>);

enum MatcherKind {
    Include(Gitignore),
    Exclude(Gitignore),
}

impl Matcher {
    pub(crate) fn new(template_config: TemplateConfig, project_dir: &Path) -> Result<Self> {
        let kind = match (&template_config.exclude, &template_config.include) {
            (None, None) => None,
            (None, Some(it)) => Some(MatcherKind::Include(Self::create_matcher(project_dir, it)?)),
            (Some(it), None) => Some(MatcherKind::Exclude(Self::create_matcher(project_dir, it)?)),
            (Some(_), Some(_)) => unreachable!(
                "BUG: template config has both include and exclude specified: {:?}",
                template_config
            ),
        };
        Ok(Self(kind))
    }

    fn create_matcher(project_dir: &Path, patterns: &[String]) -> Result<Gitignore> {
        let mut builder = GitignoreBuilder::new(project_dir);
        for rule in patterns {
            builder.add_line(None, rule)?;
        }
        Ok(builder.build()?)
    }

    pub(crate) fn should_include(&self, relative_path: &Path) -> bool {
        // "Include" and "exclude" options are mutually exclusive.
        // if no include is made, we will default to ignore_exclude
        // which if there is no options, matches everything
        match &self.0 {
            Some(MatcherKind::Exclude(it)) => {
                !it.matched_path_or_any_parents(relative_path, /* is_dir */ false)
                    .is_ignore()
            }
            Some(MatcherKind::Include(it)) => {
                it.matched_path_or_any_parents(relative_path, /* is_dir */ false)
                    .is_ignore()
            }
            None => true,
        }
    }
}
