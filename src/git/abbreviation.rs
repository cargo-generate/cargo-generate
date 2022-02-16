pub fn abbreviated_git_url_to_full_remote(git: impl AsRef<str>) -> String {
    let git = git.as_ref();
    if git.len() >= 3 {
        match &git[..3] {
            "gl:" => format!("https://gitlab.com/{}.git", &git[3..]),
            "bb:" => format!("https://bitbucket.org/{}.git", &git[3..]),
            "gh:" => format!("https://github.com/{}.git", &git[3..]),
            _ => git.to_owned(),
        }
    } else {
        git.to_owned()
    }
}
