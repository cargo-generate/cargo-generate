//! Input from user but after parse

use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::absolute_path::AbsolutePathExt;
use console::style;

use crate::{app_config::AppConfig, template_variables::CrateType, GenerateArgs, Vcs};
use log::warn;

#[derive(Debug)]
#[cfg(test)]
pub struct UserParsedInputBuilder {
    subject: UserParsedInput,
}

#[cfg(test)]
impl UserParsedInputBuilder {
    #[cfg(test)]
    pub(crate) fn for_testing() -> Self {
        use crate::TemplatePath;
        Self {
            subject: UserParsedInput::try_from_args_and_config(
                AppConfig::default(),
                &GenerateArgs {
                    destination: Some(Path::new("/tmp/dest/").to_path_buf()),
                    template_path: TemplatePath {
                        path: Some("/tmp".to_string()),
                        ..TemplatePath::default()
                    },
                    ..GenerateArgs::default()
                },
            ),
        }
    }

    pub const fn with_force(mut self) -> Self {
        self.subject.force = true;
        self
    }

    pub fn build(self) -> UserParsedInput {
        self.subject
    }
}

// Contains parsed information from user.
#[derive(Debug)]
pub struct UserParsedInput {
    name: Option<String>,

    // from where clone or copy template?
    template_location: TemplateLocation,

    destination: PathBuf,

    // if template_location contains many templates user already specified one
    subfolder: Option<String>,
    // all values that user defined through:
    // 1. environment variables
    // 2. configuration file
    // 3. cli arguments --define
    template_values: HashMap<String, toml::Value>,

    vcs: Vcs,
    pub init: bool,
    overwrite: bool,
    crate_type: CrateType,
    allow_commands: bool,
    silent: bool,
    force: bool,
    test: bool,
    force_git_init: bool,
    //TODO:
    // 1. This structure should be used instead of args
    // 2. This struct can contains internally args and app_config to not confuse
    //    other developer with parsing configuration and args by themself
}

impl UserParsedInput {
    /// Try create `UserParsedInput` reading in order [`AppConfig`] and [`Args`]
    ///
    /// # Panics
    /// This function assume that Args and AppConfig are verified earlier and are logically correct
    /// For example if both `--git` and `--path` are set this function will panic
    pub fn try_from_args_and_config(app_config: AppConfig, args: &GenerateArgs) -> Self {
        const DEFAULT_VCS: Vcs = Vcs::Git;

        let destination = args
            .destination
            .as_ref()
            .map(|p| {
                p.as_absolute()
                    .expect("cannot get the absolute path of the destination folder")
                    .to_path_buf()
            })
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));

        let cwd = env::current_dir().unwrap_or_else(|_| ".".into());

        let mut default_values = app_config.values.clone().unwrap_or_default();

        let ssh_identity = app_config
            .defaults
            .as_ref()
            .and_then(|dcfg| dcfg.ssh_identity.clone())
            .or_else(|| {
                args.ssh_identity.as_ref().cloned().or_else(|| {
                    app_config
                        .defaults
                        .as_ref()
                        .and_then(|defaults| defaults.ssh_identity.clone())
                })
            });

        // --git
        if let Some(git_url) = args.template_path.git() {
            let source = crate::template_source::TemplateSource::classify(
                git_url.as_ref(),
                &app_config,
                &cwd,
            );
            let clone_opts = clone_opts_from_args(args, ssh_identity.clone());
            return Self {
                name: args.name.clone(),
                template_location: source.into_git_template_location(&clone_opts),
                subfolder: args
                    .template_path
                    .subfolder()
                    .map(|s| s.as_ref().to_owned()),
                template_values: default_values,
                vcs: args.vcs.unwrap_or(DEFAULT_VCS),
                init: args.init,
                overwrite: args.overwrite,
                crate_type: CrateType::from(args),
                allow_commands: args.allow_commands,
                silent: args.silent,
                destination,
                force: args.force,
                test: args.template_path.test,
                force_git_init: args.force_git_init,
            };
        }

        // --path
        if let Some(path) = args.template_path.path() {
            return Self {
                name: args.name.clone(),
                template_location: path.as_ref().into(),
                subfolder: args
                    .template_path
                    .subfolder()
                    .map(|s| s.as_ref().to_owned()),
                template_values: default_values,
                vcs: args.vcs.unwrap_or(DEFAULT_VCS),
                init: args.init,
                overwrite: args.overwrite,
                crate_type: CrateType::from(args),
                allow_commands: args.allow_commands,
                silent: args.silent,
                destination,
                force: args.force,
                test: args.template_path.test,
                force_git_init: args.force_git_init,
            };
        }

        // check if favorite is favorite configuration
        let fav_name = args.template_path.any_path();

        if let Some(fav_cfg) = app_config.get_favorite_cfg(fav_name) {
            assert!(fav_cfg.git.is_none() || fav_cfg.path.is_none());

            let temp_location = fav_cfg.git.as_ref().map_or_else(
                || fav_cfg.path.as_ref().map(TemplateLocation::from).unwrap(),
                |git_url| {
                    let branch = args
                        .template_path
                        .branch()
                        .map(|s| s.as_ref().to_owned())
                        .or_else(|| fav_cfg.branch.clone());
                    let tag = args
                        .template_path
                        .tag()
                        .map(|s| s.as_ref().to_owned())
                        .or_else(|| fav_cfg.tag.clone());
                    let revision = args
                        .template_path
                        .revision()
                        .map(|s| s.as_ref().to_owned())
                        .or_else(|| fav_cfg.revision.clone());
                    let git_user_input = GitUserInput::new(
                        git_url,
                        branch.as_ref(),
                        tag.as_ref(),
                        revision.as_ref(),
                        ssh_identity,
                        None,
                        args.force_git_init,
                        args.skip_submodules,
                    );

                    TemplateLocation::from(git_user_input)
                },
            );

            if let Some(fav_default_values) = &fav_cfg.values {
                default_values.extend(fav_default_values.clone());
            }

            return Self {
                name: args.name.clone(),
                template_location: temp_location,
                subfolder: args
                    .template_path
                    .subfolder()
                    .map(|s| s.as_ref().to_owned())
                    .or_else(|| fav_cfg.subfolder.clone()),
                template_values: default_values,
                vcs: args.vcs.or(fav_cfg.vcs).unwrap_or(DEFAULT_VCS),
                init: args
                    .init
                    .then_some(true)
                    .or(fav_cfg.init)
                    .unwrap_or_default(),
                overwrite: args
                    .overwrite
                    .then_some(true)
                    .or(fav_cfg.overwrite)
                    .unwrap_or_default(),
                crate_type: CrateType::from(args),
                allow_commands: args.allow_commands,
                silent: args.silent,
                destination,
                force: args.force,
                test: args.template_path.test,
                force_git_init: args.force_git_init,
            };
        }

        // there is no specified favorite in configuration
        // auto_path with no configured favorite name → classify it
        let source = crate::template_source::TemplateSource::classify(fav_name, &app_config, &cwd);
        let clone_opts = clone_opts_from_args(args, ssh_identity);
        let temp_location = source.into_template_location(&clone_opts);

        // Print information about what happened (preserve the existing warn!)
        let location_msg = match &temp_location {
            TemplateLocation::Git(git_user_input) => {
                format!("git repository: {}", style(git_user_input.url()).bold())
            }
            TemplateLocation::Path(path) => {
                format!("local path: {}", style(path.display()).bold())
            }
        };
        warn!(
            "Favorite `{}` not found in config, using it as a {}",
            style(&fav_name).bold(),
            location_msg
        );

        Self {
            name: args.name.clone(),
            template_location: temp_location,
            subfolder: args
                .template_path
                .subfolder()
                .map(|s| s.as_ref().to_owned()),
            template_values: default_values,
            vcs: args.vcs.unwrap_or(DEFAULT_VCS),
            init: args.init,
            overwrite: args.overwrite,
            crate_type: CrateType::from(args),
            allow_commands: args.allow_commands,
            silent: args.silent,
            destination,
            force: args.force,
            test: args.template_path.test,
            force_git_init: args.force_git_init,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub const fn location(&self) -> &TemplateLocation {
        &self.template_location
    }

    pub fn subfolder(&self) -> Option<&str> {
        self.subfolder.as_deref()
    }

    pub const fn template_values(&self) -> &HashMap<String, toml::Value> {
        &self.template_values
    }

    pub const fn template_values_mut(&mut self) -> &mut HashMap<String, toml::Value> {
        &mut self.template_values
    }

    pub const fn vcs(&self) -> Vcs {
        self.vcs
    }

    pub const fn init(&self) -> bool {
        self.init
    }

    pub const fn overwrite(&self) -> bool {
        self.overwrite
    }

    pub const fn crate_type(&self) -> CrateType {
        self.crate_type
    }

    pub const fn allow_commands(&self) -> bool {
        self.allow_commands
    }

    pub const fn silent(&self) -> bool {
        self.silent
    }

    pub fn destination(&self) -> &Path {
        self.destination.as_path()
    }

    pub const fn force(&self) -> bool {
        self.force
    }

    pub const fn test(&self) -> bool {
        self.test
    }

    pub const fn force_git_init(&self) -> bool {
        self.force_git_init
    }
}

// Template should be cloned with git
#[derive(Debug)]
pub struct GitUserInput {
    url: String,
    branch: Option<String>,
    tag: Option<String>,
    revision: Option<String>,
    identity: Option<PathBuf>,
    gitconfig: Option<PathBuf>,
    _force_init: bool,
    pub skip_submodules: bool,
}

impl GitUserInput {
    #[allow(clippy::too_many_arguments)]
    fn new(
        url: &impl AsRef<str>,
        branch: Option<&impl AsRef<str>>,
        tag: Option<&impl AsRef<str>>,
        revision: Option<&impl AsRef<str>>,
        identity: Option<PathBuf>,
        gitconfig: Option<PathBuf>,
        force_init: bool,
        skip_submodules: bool,
    ) -> Self {
        Self {
            url: url.as_ref().to_owned(),
            branch: branch.map(|s| s.as_ref().to_owned()),
            tag: tag.map(|s| s.as_ref().to_owned()),
            revision: revision.map(|s| s.as_ref().to_owned()),
            identity,
            gitconfig,
            _force_init: force_init,
            skip_submodules,
        }
    }

    /// Build a `GitUserInput` from a resolved URL and the cargo-generate
    /// clone options. Used by `TemplateSource::into_template_location`.
    pub fn with_url_and_clone_opts(
        url: String,
        opts: &crate::template_source::CloneOptions,
    ) -> Self {
        Self::new(
            &url,
            opts.branch.as_ref(),
            opts.tag.as_ref(),
            opts.revision.as_ref(),
            opts.ssh_identity.clone(),
            opts.gitconfig.clone(),
            opts.force_git_init,
            opts.skip_submodules,
        )
    }

    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    pub fn identity(&self) -> Option<&Path> {
        self.identity.as_deref()
    }

    pub fn gitconfig(&self) -> Option<&Path> {
        self.gitconfig.as_deref()
    }
}

fn clone_opts_from_args(
    args: &GenerateArgs,
    ssh_identity: Option<PathBuf>,
) -> crate::template_source::CloneOptions {
    crate::template_source::CloneOptions {
        branch: args.template_path.branch().map(|s| s.as_ref().to_owned()),
        tag: args.template_path.tag().map(|s| s.as_ref().to_owned()),
        revision: args.template_path.revision().map(|s| s.as_ref().to_owned()),
        ssh_identity,
        gitconfig: args.gitconfig.clone(),
        force_git_init: args.force_git_init,
        skip_submodules: args.skip_submodules,
    }
}

// Distinguish between plain copy and clone
#[derive(Debug)]
pub enum TemplateLocation {
    Git(GitUserInput),
    Path(PathBuf),
}

impl From<GitUserInput> for TemplateLocation {
    fn from(source: GitUserInput) -> Self {
        Self::Git(source)
    }
}

impl<T> From<T> for TemplateLocation
where
    T: AsRef<Path>,
{
    fn from(source: T) -> Self {
        Self::Path(PathBuf::from(source.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a `GenerateArgs` with `--git <value>` and resolve via `try_from_args_and_config`,
    /// returning the resolved git URL from the resulting `TemplateLocation`.
    fn resolve_git_flag(git_value: &str) -> String {
        let args = GenerateArgs {
            destination: Some(std::path::PathBuf::from("/tmp")),
            template_path: crate::TemplatePath {
                git: Some(git_value.to_owned()),
                ..crate::TemplatePath::default()
            },
            ..GenerateArgs::default()
        };
        let parsed = UserParsedInput::try_from_args_and_config(AppConfig::default(), &args);
        match parsed.location() {
            TemplateLocation::Git(git) => git.url().to_owned(),
            TemplateLocation::Path(p) => panic!("expected Git location, got Path: {p:?}"),
        }
    }

    #[test]
    fn git_flag_full_url() {
        // cargo generate --git https://github.com/username-on-github/mytemplate.git
        assert_eq!(
            resolve_git_flag("https://github.com/username-on-github/mytemplate.git"),
            "https://github.com/username-on-github/mytemplate.git"
        );
    }

    #[test]
    fn git_flag_org_repo_shorthand() {
        // cargo generate --git username-on-github/mytemplate
        assert_eq!(
            resolve_git_flag("username-on-github/mytemplate"),
            "https://github.com/username-on-github/mytemplate.git"
        );
    }

    #[test]
    fn git_flag_gh_prefix() {
        // cargo generate --git gh:username-on-github/mytemplate
        assert_eq!(
            resolve_git_flag("gh:username-on-github/mytemplate"),
            "https://github.com/username-on-github/mytemplate.git"
        );
    }

    #[test]
    fn git_flag_gl_prefix() {
        // cargo generate --git gl:username-on-gitlab/mytemplate
        assert_eq!(
            resolve_git_flag("gl:username-on-gitlab/mytemplate"),
            "https://gitlab.com/username-on-gitlab/mytemplate.git"
        );
    }

    #[test]
    fn git_flag_bb_prefix() {
        // cargo generate --git bb:username-on-bitbucket/mytemplate
        assert_eq!(
            resolve_git_flag("bb:username-on-bitbucket/mytemplate"),
            "https://bitbucket.org/username-on-bitbucket/mytemplate.git"
        );
    }

    #[test]
    fn git_flag_sr_prefix() {
        // cargo generate --git sr:username-on-sourcehut/mytemplate
        assert_eq!(
            resolve_git_flag("sr:username-on-sourcehut/mytemplate"),
            "https://git.sr.ht/~username-on-sourcehut/mytemplate"
        );
    }

    #[test]
    fn git_flag_relative_path_resolves_to_local_directory() {
        // When --git receives a relative path like `./example-templates/hooks`
        // that exists as a local directory, it must remain a Git location so
        // that branch/tag/ssh-identity options are honoured (git clone accepts
        // file-system paths). It must NOT be expanded to a remote URL.
        // `cargo test` sets cwd to the crate root, so this path resolves.
        let args = GenerateArgs {
            template_path: crate::TemplatePath {
                git: Some("./example-templates/hooks".to_owned()),
                ..crate::TemplatePath::default()
            },
            ..GenerateArgs::default()
        };

        let parsed = UserParsedInput::try_from_args_and_config(AppConfig::default(), &args);

        match parsed.location() {
            TemplateLocation::Git(git) => {
                assert!(
                    git.url().ends_with("example-templates/hooks"),
                    "expected url ending in example-templates/hooks, got {}",
                    git.url()
                );
            }
            TemplateLocation::Path(p) => panic!("expected Git location, got Path: {p:?}"),
        }
    }

    #[test]
    fn git_flag_local_path_takes_precedence_over_org_repo() {
        // When --git receives a value matching the org/repo shape that also
        // exists as a local directory, it must be kept as a Git location rather
        // than expanded to github. `example-templates/hooks` doubles as a real
        // path under the crate root and a valid `org/repo` shape.
        let args = GenerateArgs {
            template_path: crate::TemplatePath {
                git: Some("example-templates/hooks".to_owned()),
                ..crate::TemplatePath::default()
            },
            ..GenerateArgs::default()
        };

        let parsed = UserParsedInput::try_from_args_and_config(AppConfig::default(), &args);
        match parsed.location() {
            TemplateLocation::Git(git) => {
                assert!(
                    git.url().ends_with("example-templates/hooks"),
                    "expected url ending in example-templates/hooks, got {}",
                    git.url()
                );
            }
            TemplateLocation::Path(p) => panic!("expected Git location, got Path: {p:?}"),
        }
    }
}
