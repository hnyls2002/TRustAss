pub mod checker;
pub mod file_sync;
pub mod file_tree;
pub mod file_watcher;

use std::path::PathBuf;

use crate::{config::TMP_PATH, info};

use self::file_tree::FileTree;

pub use std::io::Result as IoResult;

pub struct Replica<'a> {
    pub port: u16,
    pub prefix: PathBuf,
    pub online_list: Vec<FileTree<'a>>,
}

impl<'a> Replica<'a> {
    pub fn to_absolute(&self, relative: &PathBuf) -> IoResult<PathBuf> {
        let mut ret = self.prefix.clone();
        ret.push(relative);
        ret.canonicalize()
    }

    pub fn to_relative(&self, absolute: &PathBuf) -> IoResult<PathBuf> {
        if let Ok(res) = absolute.clone().strip_prefix(&self.prefix) {
            return Ok(res.to_path_buf());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not a relative path",
        ))
    }

    pub fn check_exist(&self, relative: &PathBuf) -> bool {
        let mut path = self.prefix.clone();
        path.push(relative);
        path.exists()
    }

    pub fn check_is_dir(&self, relative: &PathBuf) -> bool {
        let mut path = self.prefix.clone();
        path.push(relative);
        path.is_dir()
    }
}

impl<'a> Replica<'a> {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            prefix: PathBuf::from(format!("{}{}", TMP_PATH, port)),
            online_list: Vec::new(),
        }
    }

    // make all the exist file tree online
    pub fn initialize_from_exist(&mut self) -> IoResult<()> {
        let file_list = std::fs::read_dir(&self.prefix)?;
        for res in file_list {
            let path = res.unwrap().path();
            if path.is_dir() {
                info!(
                    "Dir found : \"{}\", Port id: {}",
                    self.to_relative(&path).unwrap().display(),
                    self.port
                );
            } else {
                info!(
                    "file found : \"{}\", Port id: {}",
                    self.to_relative(&path).unwrap().display(),
                    self.port
                );
            }
            let mut file_tree = FileTree::new_from_exist(&self, &path);
            file_tree.organize();
            file_tree.tree();
        }
        Ok(())
    }

    pub fn online_one(&mut self) -> IoResult<()> {
        todo!();
        Ok(())
    }

    pub fn clear() {
        todo!();
    }
}
