use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use clap::Parser;

use crate::git;

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
pub enum Cli {
    #[clap(name = "generate", visible_alias = "gen")]
    Generate(Args),
}

#[derive(Debug, Parser)]
pub struct Args {
    /// List defined favorite templates from the config
    #[clap(long,
        conflicts_with_all(&[
            "git",
            "subfolder",
            "path",
            "branch",
            "name",
            "force",
            "template-values-file",
            "silent",
            "vcs",
            "lib",
            "bin",
            "ssh-identity",
            "define",
            "init",
        ])
    )]
    pub list_favorites: bool,

    #[clap(flatten)]
    pub template_path: TemplatePath,

    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[clap(long, short)]
    pub name: Option<String>,

    /// Don't convert the project name to kebab-case before creating the directory.
    /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
    #[clap(long, short)]
    pub force: bool,

    /// Enables more verbose output.
    #[clap(long, short)]
    pub verbose: bool,

    /// Pass template values through a file
    /// Values should be in the format `key=value`, one per line
    #[clap(long)]
    pub template_values_file: Option<String>,

    /// If silent mode is set all variables will be
    /// extracted from the template_values_file.
    /// If a value is missing the project generation will fail
    #[clap(long, short, requires("name"))]
    pub silent: bool,

    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
    #[clap(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Specify the VCS used to initialize the generated template.
    #[clap(long, default_value = "git")]
    pub vcs: Vcs,

    /// Populates a template variable `crate_type` with value `"lib"`
    #[clap(long, conflicts_with("bin"))]
    pub lib: bool,

    /// Populates a template variable `crate_type` with value `"bin"`
    #[clap(long, conflicts_with("lib"))]
    pub bin: bool,

    /// Use a different ssh identity
    #[clap(short = 'i', long = "identity", parse(from_os_str))]
    pub ssh_identity: Option<PathBuf>,

    /// Define a value for use during template expansion
    #[clap(long, short, number_of_values = 1)]
    pub define: Vec<String>,

    /// Generate the template directly into the current dir. No subfolder will be created and no vcs is initialized.
    #[clap(long)]
    pub init: bool,

    /// Will enforce a fresh git init on the generated project
    #[clap(long)]
    pub force_git_init: bool,

    /// Allows running system commands without being prompted.
    /// Warning: Setting this flag will enable the template to run arbitrary system commands without user confirmation.
    /// Use at your own risk and be sure to review the template code beforehand.
    #[clap(short, long)]
    pub allow_commands: bool,
}

#[derive(Debug, Parser, Default)]
pub struct TemplatePath {
    /// Auto attempt to use as either `--git`, `--path` or `--favorite`.
    #[clap(index(1), required_unless_present_any(&["SpecificPath", "list-favorites"]))]
    pub auto_path: Option<String>,

    /// Specifies a subfolder within the template repository to be used as the actual template.
    #[clap(index(2))]
    pub subfolder: Option<String>,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    ///
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[clap(short, long, group("SpecificPath"))]
    pub git: Option<String>,

    /// Branch to use when installing from git
    #[clap(short, long)]
    pub branch: Option<String>,

    /// Local path to copy the template from. Can not be specified together with --git.
    #[clap(name = "path", short, long, group("SpecificPath"))]
    pub local_path: Option<String>,

    /// Generate a favorite template as defined in the config. In case the favorite is undefined,
    /// use in place of the `--git` option, otherwise specifies the subfolder
    #[clap(long, group("SpecificPath"))]
    pub favorite: Option<String>,
}

impl TemplatePath {
    pub fn git(&self) -> Option<String> {
        self.git.as_ref().cloned()
    }

    pub fn local_path(&self) -> Option<String> {
        self.local_path.as_ref().cloned()
    }

    pub fn favorite(&self) -> Option<String> {
        self.favorite.as_ref().cloned()
    }

    pub fn auto_path(&self) -> Option<String> {
        if self.git.is_some() || self.local_path.is_some() || self.favorite.is_some() {
            None
        } else {
            self.auto_path.clone()
        }
    }

    pub fn branch(&self) -> Option<String> {
        self.branch.clone()
    }

    pub fn subfolder(&self) -> Option<String> {
        if self.git.is_some() || self.local_path.is_some() || self.favorite.is_some() {
            self.auto_path.clone()
        } else {
            self.subfolder.clone()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Vcs {
    None,
    Git,
}

impl FromStr for Vcs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NONE" => Ok(Self::None),
            "GIT" => Ok(Self::Git),
            _ => Err(anyhow!("Must be one of 'git' or 'none'")),
        }
    }
}

impl Vcs {
    pub fn initialize(&self, project_dir: &Path, branch: String, force: bool) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Git => git::init(project_dir, &branch, force)
                .map(|_| ())
                .map_err(anyhow::Error::from),
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[cfg(test)]
mod tests {
    mod list_favorites {
        use clap::Parser;

        use crate::*;

        type Result = anyhow::Result<()>;

        #[test]
        fn can_be_specified_alone() -> Result {
            let args = Args::try_parse_from(&["cmd", "--list-favorites"])?;
            assert!(args.list_favorites);
            assert!(args.template_path.git().is_none());
            assert!(args.template_path.local_path().is_none());
            assert!(args.template_path.favorite().is_none());
            assert!(args.template_path.auto_path().is_none());
            Ok(())
        }

        #[test]
        fn accepts_auto_path() -> Result {
            let args = Args::try_parse_from(&["cmd", "--list-favorites", "autopath"])?;
            assert!(matches!(args.template_path.auto_path(), Some(s) if s=="autopath"));
            Ok(())
        }

        #[test]
        #[should_panic]
        fn conflicts_with_git() {
            Args::try_parse_from(&["cmd", "--list-favorites", "--git", "git"]).unwrap();
        }

        #[test]
        fn accepts_favorite_path() -> Result {
            let args = Args::try_parse_from(&["cmd", "--list-favorites", "--favorite", "fav"])?;
            assert!(matches!(args.template_path.favorite(), Some(s) if s=="fav"));
            Ok(())
        }

        #[test]
        #[should_panic]
        fn conflicts_with_path() {
            Args::try_parse_from(&["cmd", "--list-favorites", "--path", "path"]).unwrap();
        }
    }

    mod template_path_specified_as {
        use clap::Parser;

        use crate::*;

        type Result = anyhow::Result<()>;

        #[test]
        fn auto_path() -> Result {
            let args = Args::try_parse_from(&["cmd", "mypath"])?;
            assert!(matches!(args.template_path.auto_path(), Some(s) if s=="mypath"));
            assert!(args.template_path.branch().is_none());
            assert!(args.template_path.subfolder().is_none());
            Ok(())
        }

        #[test]
        fn git_folder() -> Result {
            let args = Args::try_parse_from(&["cmd", "--git", "git"])?;
            assert!(matches!(args.template_path.git(), Some(s) if s=="git"));
            assert!(args.template_path.branch().is_none());
            assert!(args.template_path.subfolder().is_none());
            Ok(())
        }

        #[test]
        fn path_folder() -> Result {
            let args = Args::try_parse_from(&["cmd", "--path", "path"])?;
            assert!(matches!(args.template_path.local_path(), Some(s) if s=="path"));
            assert!(args.template_path.branch().is_none());
            assert!(args.template_path.subfolder().is_none());
            Ok(())
        }

        #[test]
        fn favorites_folder() -> Result {
            let args = Args::try_parse_from(&["cmd", "--favorite", "fav"])?;
            assert!(matches!(args.template_path.favorite(), Some(s) if s=="fav"));
            assert!(args.template_path.branch().is_none());
            assert!(args.template_path.subfolder().is_none());
            Ok(())
        }

        #[test]
        fn auto_path_with_subfolder() -> Result {
            let args = Args::try_parse_from(&["cmd", "mypath", "sub"])?;
            assert!(matches!(args.template_path.auto_path(), Some(s) if s=="mypath"));
            assert!(args.template_path.branch().is_none());
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn git_folder_with_subfolder() -> Result {
            let args = Args::try_parse_from(&["cmd", "--git", "git", "sub"])?;
            assert!(matches!(args.template_path.git(), Some(s) if s=="git"));
            assert!(args.template_path.branch().is_none());
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn path_folder_with_subfolder() -> Result {
            let args = Args::try_parse_from(&["cmd", "--path", "path", "sub"])?;
            assert!(matches!(args.template_path.local_path(), Some(s) if s=="path"));
            assert!(args.template_path.branch().is_none());
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn favorites_folder_with_subfolder() -> Result {
            let args = Args::try_parse_from(&["cmd", "--favorite", "fav", "sub"])?;
            assert!(matches!(args.template_path.favorite(), Some(s) if s=="fav"));
            assert!(args.template_path.branch().is_none());
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn auto_path_with_subfolder_and_branch() -> Result {
            let args = Args::try_parse_from(&["cmd", "--branch", "branch", "mypath", "sub"])?;
            assert!(matches!(args.template_path.auto_path(), Some(s) if s=="mypath"));
            assert!(matches!(args.template_path.branch(), Some(s) if s=="branch"));
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn git_folder_with_subfolder_and_branch() -> Result {
            let args = Args::try_parse_from(&["cmd", "--branch", "branch", "--git", "git", "sub"])?;
            assert!(matches!(args.template_path.git(), Some(s) if s=="git"));
            assert!(matches!(args.template_path.branch(), Some(s) if s=="branch"));
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn path_folder_with_subfolder_and_branch() -> Result {
            let args =
                Args::try_parse_from(&["cmd", "--branch", "branch", "--path", "path", "sub"])?;
            assert!(matches!(args.template_path.local_path(), Some(s) if s=="path"));
            assert!(matches!(args.template_path.branch(), Some(s) if s=="branch"));
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }

        #[test]
        fn favorites_folder_with_subfolder_and_branch() -> Result {
            let args =
                Args::try_parse_from(&["cmd", "--branch", "branch", "--favorite", "fav", "sub"])?;
            assert!(matches!(args.template_path.favorite(), Some(s) if s=="fav"));
            assert!(matches!(args.template_path.branch(), Some(s) if s=="branch"));
            assert!(matches!(args.template_path.subfolder(), Some(s) if s=="sub"));
            Ok(())
        }
    }
}
