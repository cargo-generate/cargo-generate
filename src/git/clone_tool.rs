use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::Context;
use anyhow::Result;
use auth_git2::GitAuthenticator;
use console::style;
use git2::{build::RepoBuilder, Config, FetchOptions, ProxyOptions, Repository};
use log::debug;

fn interactive_auth_allowed() -> bool {
    std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
}

fn is_http_repo_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn should_limit_fetch_depth(url: &str, requires_full_history: bool) -> bool {
    is_http_repo_url(url) && !requires_full_history
}

use crate::emoji::WRENCH;

use super::gitconfig;
use super::gitconfig::find_gitconfig;
use super::utils;

pub struct RepoCloneBuilder<'cb> {
    builder: RepoBuilder<'cb>,
    authenticator: GitAuthenticator,
    url: String,
    skip_submodules: bool,
    destination_path: Option<PathBuf>,
    tag_or_revision: Option<String>,
    requires_full_history: bool,
    gitconfig: Option<Config>,
    interactive: bool,
}

impl<'cb> RepoCloneBuilder<'cb> {
    pub fn new(url: &str) -> Self {
        let interactive = interactive_auth_allowed();
        #[cfg(windows)]
        let authenticator = GitAuthenticator::default().try_ssh_agent(true);
        #[cfg(not(windows))]
        let authenticator = GitAuthenticator::default()
            .try_ssh_agent(true)
            .add_default_ssh_keys()
            .prompt_ssh_key_password(interactive)
            .try_password_prompt(if interactive { 3 } else { 0 });

        Self {
            builder: RepoBuilder::new(),
            authenticator,
            url: url.to_owned(),
            skip_submodules: false,
            destination_path: None,
            tag_or_revision: None,
            requires_full_history: false,
            gitconfig: None,
            interactive,
        }
    }

    pub const fn with_submodules(mut self, with_submodules: bool) -> Self {
        self.skip_submodules = !with_submodules;
        self
    }

    /// Might alter the url via gitconfig "instead url" configuration
    pub fn with_gitconfig(mut self, gitcfg: Option<&Path>) -> Result<Self> {
        if let Some(gitconfig) = gitcfg
            .map(|p| p.to_owned())
            .or_else(|| find_gitconfig().map_or(None, |gitconfig| gitconfig))
        {
            self.gitconfig = Some(Config::open(gitconfig.as_path())?);

            if let Some(url) = gitconfig::resolve_instead_url(&self.url, gitconfig)? {
                debug!("{} gitconfig 'insteadOf' lead to this url: {}", WRENCH, url);
                self.url = url;
            }
        }

        Ok(self)
    }

    /// SSH key files are used for authentication if provided.
    /// If a password is required, the user will be prompted.
    /// If the password is incorrect, the user will be prompted 3 times in total.
    pub fn with_ssh_identity(mut self, identity_path: Option<&Path>) -> Result<Self> {
        if let Some(identity_path) = identity_path {
            let identity_path = utils::canonicalize_path(identity_path)?;
            log::info!(
                "{} `{}` {}",
                style("Using private key:").bold(),
                style(format_args!("{}", identity_path.display()))
                    .bold()
                    .yellow(),
                style("for git-ssh checkout").bold()
            );

            let interactive = self.interactive;
            self.authenticator = self
                .authenticator
                .add_ssh_key_from_file(identity_path, None)
                .try_password_prompt(if interactive { 3 } else { 0 })
                .prompt_ssh_key_password(interactive);
        }

        Ok(self)
    }

    pub fn with_branch(mut self, branch: Option<&str>) -> Self {
        if let Some(branch) = branch {
            self.builder.branch(branch);
        }

        self
    }

    /// Ensures a specific tag is cloned
    /// Note: this overrides the revision if set
    pub fn with_tag(mut self, tag: Option<&str>) -> Self {
        if let Some(tag) = tag {
            self.tag_or_revision = Some(tag.to_owned());
            self.requires_full_history = false;
        }

        self
    }

    /// Ensures a specific revision is cloned
    /// Note: this overrides the tag if set
    pub fn with_revision(mut self, revision: Option<&str>) -> Self {
        if let Some(revision) = revision {
            self.tag_or_revision = Some(revision.to_owned());
            self.requires_full_history = true;
        }

        self
    }

    pub fn with_destination(mut self, destination_path: impl AsRef<Path>) -> Result<Self> {
        self.destination_path = Some(utils::canonicalize_path(destination_path.as_ref())?);

        Ok(self)
    }

    /// creates a Result to the final GitCloneCmd wrapper
    pub fn build(self) -> Result<GitCloneCmd<'cb>> {
        if self.destination_path.is_none() {
            return Err(anyhow::anyhow!("Destination path is not set"));
        };

        Ok(GitCloneCmd { builder: self })
    }
}

pub struct GitCloneCmd<'cb> {
    builder: RepoCloneBuilder<'cb>,
}

impl GitCloneCmd<'_> {
    fn do_clone_repo(self) -> Result<Repository> {
        let gitconfig = self
            .builder
            .gitconfig
            .map(Result::Ok)
            .unwrap_or_else(git2::Config::open_default)?;
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = git2::RemoteCallbacks::new();

        callbacks.credentials(self.builder.authenticator.credentials(&gitconfig));
        fetch_options.remote_callbacks(callbacks);

        let url = self.builder.url.clone();

        let is_ssh_repo = url.starts_with("ssh}://") || url.starts_with("git@");
        let is_http_repo = is_http_repo_url(&url);

        if should_limit_fetch_depth(&url, self.builder.requires_full_history) {
            let mut proxy_options = ProxyOptions::new();
            proxy_options.auto();

            fetch_options.proxy_options(proxy_options);
            fetch_options.depth(1);
        }

        if is_ssh_repo || is_http_repo {
            fetch_options.download_tags(git2::AutotagOption::All);
        }

        let mut builder = self.builder.builder;
        builder.fetch_options(fetch_options);

        let repository = builder
            .clone(&url, &self.builder.destination_path.unwrap())
            .context("Please check if the Git user / repository exists.")?;

        if let Some(tag_or_revision) = &self.builder.tag_or_revision {
            let (object, reference) = repository.revparse_ext(tag_or_revision)?;
            repository.checkout_tree(&object, None)?;
            reference.map_or_else(
                || repository.set_head_detached(object.id()),
                |gref| repository.set_head(gref.name().unwrap()),
            )?
        }

        Ok(repository)
    }

    /// Clones the repository with submodules
    pub fn do_clone(self) -> Result<Repository> {
        let authenticator = Clone::clone(&self.builder.authenticator);
        let skip_submodules = self.builder.skip_submodules;
        let repo = self.do_clone_repo()?;

        if skip_submodules {
            return Ok(repo);
        }

        let config = repo.config()?;

        for mut sub in repo.submodules()? {
            let mut proxy_options = ProxyOptions::new();
            proxy_options.auto();

            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(authenticator.credentials(&config));

            let mut fetch_options = FetchOptions::new();
            fetch_options.proxy_options(proxy_options);
            fetch_options.remote_callbacks(callbacks);

            let mut update_options = git2::SubmoduleUpdateOptions::new();
            update_options.fetch(fetch_options);
            sub.update(true, Some(&mut update_options))?;
        }

        Ok(repo)
    }
}

#[cfg(test)]
mod tests {
    use super::{should_limit_fetch_depth, RepoCloneBuilder};

    #[test]
    fn http_clones_are_shallow_by_default() {
        assert!(should_limit_fetch_depth(
            "https://github.com/example/template",
            false
        ));
    }

    #[test]
    fn revision_clones_skip_shallow_http_fetch() {
        assert!(!should_limit_fetch_depth(
            "https://github.com/example/template",
            true
        ));
    }

    #[test]
    fn non_http_clones_do_not_set_fetch_depth() {
        assert!(!should_limit_fetch_depth("git@example.com:repo.git", false));
    }

    #[test]
    fn revision_clones_require_full_history() {
        let builder =
            RepoCloneBuilder::new("https://github.com/example/template").with_revision(Some("abc"));

        assert!(builder.requires_full_history);
    }

    #[test]
    fn tag_clones_can_stay_shallow() {
        let builder =
            RepoCloneBuilder::new("https://github.com/example/template").with_tag(Some("v1.0.0"));

        assert!(!builder.requires_full_history);
    }

    #[test]
    fn tag_overrides_previous_revision_history_requirement() {
        let builder = RepoCloneBuilder::new("https://github.com/example/template")
            .with_revision(Some("abc"))
            .with_tag(Some("v1.0.0"));

        assert!(!builder.requires_full_history);
    }
}
