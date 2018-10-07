use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Clone, Debug)]
pub struct Project {
    pub root: PathBuf,
}

impl Project {
    pub fn read(&self, path: &str) -> String {
        let mut ret = String::new();
        File::open(self.root.join(path))
            .expect(&format!("couldn't open file {:?}", self.root.join(path)))
            .read_to_string(&mut ret)
            .expect(&format!("couldn't read file {:?}", self.root.join(path)));
        return ret;
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.root));
        drop(fs::remove_dir(&self.root));
    }
}
