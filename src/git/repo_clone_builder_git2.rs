use crate::git::gitconfig;
use crate::utils::canonicalize_path;
use console::style;
use std::path::Path;

type Git2Result<T> = anyhow::Result<T, git2::Error>;

// cargo-generate (as application) want from git module:
// 1. cloning remote
// 2. initialize freshly generated template
// 3. remove history from cloned template

// Assumptions:
// * `--git <url>` should only be parse in the same way as `git clone <url>` would
// * submodules should be clone by default
// * `.git` should be removed to make clear repository
// * if `<url>` is the local path on system the clone should also be done the same way as `git clone` there is `--path`
//    for different behavior

// basically we want to call:
// git clone --recurse-submodules --depth 1 --branch <branch> <url> <tmp_dir>
struct RepoCloneBuilder<'cb> {
    builder: git2::build::RepoBuilder<'cb>,
    authenticator: auth_git2::GitAuthenticator,
    url: String,
}

impl<'cb> RepoCloneBuilder<'cb> {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let url = gitconfig::find_gitconfig()?.map_or_else(
            || url.to_owned(),
            |gitcfg| {
                gitconfig::resolve_instead_url(url, gitcfg)
                    .expect("correct configuration")
                    .unwrap_or_else(|| url.to_owned())
            },
        );

        Ok(Self {
            builder: git2::build::RepoBuilder::new(),
            authenticator: auth_git2::GitAuthenticator::default(),
            url,
        })
    }

    pub fn new_with(
        url: &str,
        branch: Option<&str>,
        identity_path: Option<&Path>,
    ) -> anyhow::Result<Self> {
        let mut builder = Self::new(url)?;
        if let Some(branch) = branch {
            builder.set_branch(branch);
        }

        if let Some(identity_path) = identity_path {
            builder.set_identity(identity_path)?;
        }

        Ok(builder)
    }

    pub fn set_identity(&mut self, identity_path: &Path) -> anyhow::Result<()> {
        let identity_path = canonicalize_path(identity_path)?;
        log::info!(
            "{} `{}` {}",
            style("Using private key:").bold(),
            style(format_args!("{}", identity_path.display()))
                .bold()
                .yellow(),
            style("for git-ssh checkout").bold()
        );
        self.authenticator = auth_git2::GitAuthenticator::new_empty()
            .add_ssh_key_from_file(identity_path, None)
            .prompt_ssh_key_password(true);
        Ok(())
    }

    pub fn set_branch(&mut self, branch: &str) {
        self.builder.branch(branch);
    }

    fn clone(self, dest_path: &Path) -> anyhow::Result<git2::Repository> {
        let config = git2::Config::open_default()?;

        let mut proxy_options = git2::ProxyOptions::new();
        proxy_options.auto();

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(self.authenticator.credentials(&config));

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.proxy_options(proxy_options);
        fetch_options.remote_callbacks(callbacks);

        let mut builder = self.builder;
        builder.fetch_options(fetch_options);

        builder
            .clone(&self.url, dest_path)
            .map_err(anyhow::Error::from)
    }

    pub fn clone_with_submodules(self, dest_path: &Path) -> anyhow::Result<git2::Repository> {
        let authenticator = Clone::clone(&self.authenticator);
        let repo = self.clone(dest_path)?;

        let config = repo.config()?;

        for mut sub in repo.submodules()? {
            let mut proxy_options = git2::ProxyOptions::new();
            proxy_options.auto();

            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(authenticator.credentials(&config));

            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.proxy_options(proxy_options);
            fetch_options.remote_callbacks(callbacks);

            let mut update_options = git2::SubmoduleUpdateOptions::new();
            update_options.fetch(fetch_options);
            sub.update(true, Some(&mut update_options))?;
        }

        Ok(repo)
    }
}
