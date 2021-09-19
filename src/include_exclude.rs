use crate::{
    config::{TemplateConfig, CONFIG_FILE_NAME},
    emoji,
};
use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

#[derive(Default)]
pub struct Matcher(Option<MatcherKind>, Vec<String>);

pub enum ShouldInclude {
    Include,
    Exclude,
    Ignore,
}

enum MatcherKind {
    Include(Gitignore),
    Exclude(Gitignore),
}

impl Matcher {
    pub(crate) fn new(
        mut template_config: &mut TemplateConfig,
        project_dir: &Path,
        permanent_excluded: &[String],
    ) -> Result<Self> {
        if template_config.include.is_some() && template_config.exclude.is_some() {
            template_config.exclude = None;
            println!(
                "{0} Your {1} contains both an include and exclude list. \
                    Only the include list will be considered. \
                    You should remove the exclude list for clarity. {0}",
                emoji::WARN,
                CONFIG_FILE_NAME
            )
        }

        let kind = match (&template_config.exclude, &template_config.include) {
            (None, None) => None,
            (None, Some(it)) => Some(MatcherKind::Include(Self::create_matcher(project_dir, it)?)),
            (Some(it), None) => Some(MatcherKind::Exclude(Self::create_matcher(project_dir, it)?)),
            (Some(_), Some(_)) => unreachable!(
                "BUG: template config has both include and exclude specified: {:?}",
                template_config
            ),
        };
        Ok(Self(kind, permanent_excluded.into()))
    }

    fn create_matcher(project_dir: &Path, patterns: &[String]) -> Result<Gitignore> {
        let mut builder = GitignoreBuilder::new(project_dir);
        for rule in patterns {
            builder.add_line(None, rule)?;
        }
        Ok(builder.build()?)
    }

    pub fn should_include(&self, relative_path: &Path) -> ShouldInclude {
        if self
            .1
            .iter()
            .any(|e| relative_path.to_str().map(|p| p == e).unwrap_or_default())
        {
            return ShouldInclude::Ignore;
        }

        // "Include" and "exclude" options are mutually exclusive.
        // if no include is made, we will default to ignore_exclude
        // which if there is no options, matches everything
        if match &self.0 {
            Some(MatcherKind::Exclude(it)) => {
                !it.matched_path_or_any_parents(relative_path, /* is_dir */ false)
                    .is_ignore()
            }
            Some(MatcherKind::Include(it)) => {
                it.matched_path_or_any_parents(relative_path, /* is_dir */ false)
                    .is_ignore()
            }
            None => true,
        } {
            ShouldInclude::Include
        } else {
            ShouldInclude::Exclude
        }
    }
}
