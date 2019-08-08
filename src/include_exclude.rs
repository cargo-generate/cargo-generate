use crate::config::TemplateConfig;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::Match;
use std::path::Path;

#[derive(Default)]
pub struct Matcher {
    template_config: Option<TemplateConfig>,
    include_matcher: Option<Gitignore>,
    exclude_matcher: Option<Gitignore>,
}

impl Matcher {
    pub fn new(
        template_config: TemplateConfig,
        project_dir: &Path,
    ) -> Result<Self, failure::Error> {
        let mut include_builder = GitignoreBuilder::new(project_dir);
        for rule in template_config.include.clone().unwrap_or_else(Vec::new) {
            include_builder.add_line(None, &rule)?;
        }
        let include_matcher = include_builder.build()?;

        let mut exclude_builder = GitignoreBuilder::new(project_dir);
        for rule in template_config.exclude.clone().unwrap_or_else(Vec::new) {
            exclude_builder.add_line(None, &rule)?;
        }
        let exclude_matcher = exclude_builder.build()?;

        Ok(Matcher {
            template_config: Some(template_config),
            include_matcher: Some(include_matcher),
            exclude_matcher: Some(exclude_matcher),
        })
    }

    pub fn should_include(&self, relative_path: &Path) -> bool {
        match self {
            Matcher {
                template_config: Some(template_config),
                include_matcher: Some(include_matcher),
                exclude_matcher: Some(exclude_matcher),
            } => {
                // "Include" and "exclude" options are mutually exclusive.
                // if no include is made, we will default to ignore_exclude
                // which if there is no options, matches everything
                if template_config.include.is_none() {
                    match exclude_matcher
                        .matched_path_or_any_parents(relative_path, /* is_dir */ false)
                    {
                        Match::None => true,
                        Match::Ignore(_) => false,
                        Match::Whitelist(_) => true,
                    }
                } else {
                    match include_matcher
                        .matched_path_or_any_parents(relative_path, /* is_dir */ false)
                    {
                        Match::None => false,
                        Match::Ignore(_) => true,
                        Match::Whitelist(_) => false,
                    }
                }
            }
            // if no template config, process all files
            _ => true,
        }
    }
}
