use anyhow::Context;
use git2::{
    build::RepoBuilder, Error, FetchOptions, ProxyOptions, Repository, RepositoryInitOptions,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

use crate::warn;

// TODO this function should be new structure that represent temporary location of cloned template
pub fn clone_git_template_into_temp(
    git: &str,
    branch: Option<&str>,
    identity: Option<PathBuf>,
) -> anyhow::Result<(TempDir, String)> {
    let git_clone_dir = tempfile::tempdir()?;

    let mut builder = GitTemplate::new(git, branch, identity)?;

    let repo = builder
        .clone_with_submodules(git_clone_dir.path().to_path_buf())
        .context("Please check if the Git user / repository exists.")?;
    let branch = get_branch_name_repo(&repo)?;

    // change from git repository into normal folder.
    degitalize_repo(git_clone_dir.path())?;

    Ok((git_clone_dir, branch))
}

type GitResult<T> = Result<T, Error>;

/// Represnets template as git repository
pub struct GitTemplate<'cb> {
    git_url: String,
    builder: RepoBuilder<'cb>,
}

impl<'cb> GitTemplate<'cb> {
    /// Create new location of source template that can be cloned.
    ///
    /// # Notet
    /// If `git_repo_loc` store path to local file repository it will be cloned without changes!
    pub fn new(
        git_url: &str,
        branch: Option<&str>,
        identity: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        // resolve rempte_like
        // 1. This can be valid git like: http://, https://, git@..
        // 2. This can be valid path to local git repository

        let mut builder = RepoBuilder::new();
        let mut fo = FetchOptions::new();

        if let Some(branch) = branch {
            builder.branch(branch);
        }

        let mut po = ProxyOptions::new();
        po.auto();
        fo.proxy_options(po);

        let cbs = cred::credentials_callbacks(identity)?;
        fo.remote_callbacks(cbs);

        builder.fetch_options(fo);

        Ok(Self {
            git_url: git_url.to_owned(),
            builder,
        })
    }

    fn clone(&mut self, dest_path: PathBuf) -> GitResult<Repository> {
        self.builder.clone(&self.git_url, &dest_path)
    }

    /// Clone repository into `dest_path` and int,update all submodules
    pub fn clone_with_submodules(&mut self, dest_path: PathBuf) -> GitResult<Repository> {
        self.clone(dest_path).and_then(|repo| {
            for mut sub in repo.submodules().unwrap() {
                sub.update(true, None)?;
            }

            Ok(repo)
        })
    }
}

//
// Helpers
//

/// thanks to @extrawurst for pointing this out
/// <https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/branch/mod.rs#L38>
fn get_branch_name_repo(repo: &Repository) -> anyhow::Result<String> {
    let iter = repo.branches(None)?;

    for b in iter {
        let b = b?;

        if b.0.is_head() {
            let name = b.0.name()?.unwrap_or("");
            return Ok(name.into());
        }
    }

    anyhow::bail!("A repo has no Head")
}

use std::ops::Sub;
use std::thread::sleep;
use std::time::Duration;

/// remove context of repository by removing `.git` from filesystem
pub fn degitalize_repo(project_dir: &Path) -> std::io::Result<()> {
    let git_dir = project_dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        let mut attempt = 0_u8;

        loop {
            attempt += 1;
            if let Err(e) = fs::remove_dir_all(&git_dir) {
                if attempt == 5 {
                    return Err(e);
                }

                if e.to_string().contains("The process cannot access the file because it is being used by another process.") {
                    let wait_for = Duration::from_secs(2_u64.pow(attempt.sub(1).into()));
                    warn!("Git history cleanup failed with a windows process blocking error. [Retry in {:?}]", wait_for);
                    sleep(wait_for);
                } else {
                    return Err(e);
                }
            } else {
                return Ok(());
            }
        }
    } else {
        //FIXME should we assume this is expected by caller?
        // panic!("tmp panic");
        Ok(())
    }
}

/// Init new repository in `project_dir` with `branch`
///
/// If `force` existing repository will be overwritten
pub fn init(project_dir: &Path, branch: &str, force: bool) -> anyhow::Result<Repository> {
    fn just_init(project_dir: &Path, branch: &str) -> anyhow::Result<Repository> {
        let mut opts = RepositoryInitOptions::new();
        opts.bare(false);
        opts.initial_head(branch);
        Repository::init_opts(project_dir, &opts).context("Couldn't init new repository")
    }

    match Repository::discover(project_dir) {
        Ok(repo) => {
            if force {
                Repository::open(project_dir).or_else(|_| just_init(project_dir, branch))
            } else {
                Ok(repo)
            }
        }
        Err(_) => just_init(project_dir, branch),
    }
}

mod cred {
    //! Dealing with credentials

    use anyhow::Context;
    use git2::{Cred, CredentialType, Error, ErrorClass, ErrorCode, RemoteCallbacks};
    use std::path::PathBuf;

    type InnerCredCallBack<'a> = dyn FnMut(&str) -> Result<Cred, Error> + 'a;

    struct PossibleCred<'a> {
        ct: CredentialType,
        cb: Box<InnerCredCallBack<'a>>,
    }

    impl<'a> PossibleCred<'a> {
        const fn credtype(&self) -> CredentialType {
            self.ct
        }
    }

    struct PossibleCreds<'a> {
        // list of available credentials in user order to try
        creds: Vec<Option<PossibleCred<'a>>>,
    }

    impl<'a> PossibleCreds<'a> {
        fn empty() -> Self {
            Self {
                creds: Vec::with_capacity(8),
            }
        }

        fn next(
            &mut self,
            _url: &str,
            username_from_url: &str,
            allowed_types: CredentialType,
        ) -> super::GitResult<Cred> {
            for opt_cert in self.creds.iter_mut() {
                let take = opt_cert
                    .as_ref()
                    .map_or(false, |cred| allowed_types.contains(cred.credtype()));

                if take {
                    return (opt_cert.take().unwrap().cb)(username_from_url);
                }
            }

            Err(Error::new(
                ErrorCode::Auth,
                ErrorClass::None,
                "no more credentials left",
            ))
        }

        fn add_cred(&mut self, cred: PossibleCred<'a>) {
            self.creds.push(Some(cred));
        }
    }

    fn ssh_dir() -> anyhow::Result<PathBuf> {
        dirs::home_dir()
            .context("unable to receive location of home dictionary")
            .and_then(|home_dir| {
                home_dir
                    .join(".ssh/")
                    .canonicalize()
                    .map_err(anyhow::Error::from)
            })
    }

    fn get_default_paths_to_keys() -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
        let list_of_def_keys_names = vec!["id_rsa", "id_ed25519"];
        let mut paths_to_existing_keys = Vec::with_capacity(list_of_def_keys_names.len());
        let ssh_dir = ssh_dir()?;

        for key_name in list_of_def_keys_names {
            let r#priv = ssh_dir.join(key_name);
            let r#pub = ssh_dir.join(format!("{}.pub", key_name));

            if r#priv.exists() && r#pub.exists() {
                paths_to_existing_keys.push((r#pub, r#priv));
            }
        }

        Ok(paths_to_existing_keys)
    }

    fn get_ssh_keys_credentials_callbacks<'a>(
        keys: Vec<(PathBuf, PathBuf)>,
    ) -> Vec<PossibleCred<'a>> {
        let mut creds = Vec::with_capacity(keys.len());
        for (publickey, privatekey) in keys {
            creds.push(PossibleCred {
                ct: CredentialType::SSH_KEY,
                cb: Box::new(move |username_from_url| {
                    Cred::ssh_key(username_from_url, Some(&publickey), &privatekey, None)
                }),
            })
        }
        creds
    }

    pub fn credentials_callbacks<'a>(
        user_identity: Option<PathBuf>,
    ) -> anyhow::Result<RemoteCallbacks<'a>> {
        let mut callbacks = RemoteCallbacks::new();

        let mut creds = PossibleCreds::empty();

        if let Some(priv_user_identity) = user_identity {
            //FIXME here we could also use `get_ssh_keys_credentials_callbacks` if we could add `.pub`
            //to private key
            let privatekey = priv_user_identity.canonicalize()?;

            creds.add_cred(PossibleCred {
                ct: CredentialType::SSH_KEY,
                cb: Box::new(move |username_from_url| {
                    Cred::ssh_key(username_from_url, None, &privatekey, None)
                }),
            })
        }

        // look for auths in ssh-agent
        creds.add_cred(PossibleCred {
            ct: CredentialType::SSH_KEY,
            cb: Box::new(Cred::ssh_key_from_agent),
        });

        //TODO catch if ssh-agent is unusable or is empty then fallback to `get_ssh_keys_credentials_callbacks()`

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            creds.next(url, username_from_url.unwrap_or("git"), allowed_types)
        });

        Ok(callbacks)
    }
}
