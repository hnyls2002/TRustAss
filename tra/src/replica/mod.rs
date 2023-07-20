pub mod checker;
pub mod file_sync;
pub mod file_tree;
pub mod file_watcher;

use std::{path::PathBuf, sync::Arc};

use crate::{config::TMP_PATH, get_res, info, MyResult};

use self::file_tree::{node::NodeStatus, FileTree};

pub struct RepMeta {
    pub port: u16,
    pub prefix: PathBuf,
}

impl RepMeta {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            prefix: PathBuf::from(format!("{}{}", TMP_PATH, port)),
        }
    }

    pub fn to_absolute(&self, relative: &PathBuf) -> PathBuf {
        let mut ret = self.prefix.clone();
        ret.push(relative);
        ret
    }

    pub fn to_relative(&self, absolute: &PathBuf) -> Option<PathBuf> {
        absolute
            .clone()
            .strip_prefix(&self.prefix)
            .map_or(None, |f| Some(f.to_path_buf()))
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

    pub fn get_status(&self, relative: &PathBuf) -> NodeStatus {
        self.check_exist(relative)
            .then(|| NodeStatus::Exist)
            .unwrap_or(NodeStatus::Deleted)
    }
}

pub struct Replica {
    pub rep_meta: Arc<RepMeta>,
    pub online_list: Vec<FileTree>,
}

impl Replica {
    pub fn new(port: u16) -> Self {
        Self {
            rep_meta: Arc::new(RepMeta::new(port)),
            online_list: Vec::new(),
        }
    }

    // make all the exist file tree online
    pub fn initialize_from_exist(&mut self) -> MyResult<()> {
        let file_list = get_res!(std::fs::read_dir(&self.rep_meta.prefix));
        for res in file_list {
            let path = res.unwrap().path();
            if path.is_dir() {
                info!(
                    "Dir found : \"{}\", Port id: {}",
                    self.rep_meta.to_relative(&path).unwrap().display(),
                    self.rep_meta.port
                );
            } else {
                info!(
                    "file found : \"{}\", Port id: {}",
                    self.rep_meta.to_relative(&path).unwrap().display(),
                    self.rep_meta.port
                );
            }
            let mut file_tree = FileTree::new_from_path(self.rep_meta.clone(), &path)?;
            file_tree.organize();
            file_tree.tree();
        }
        Ok(())
    }

    pub fn online_one(&mut self, relative_path: &PathBuf) -> MyResult<()> {
        if relative_path.components().count() != 1 {
            return Err("online file(folder) must be in the root dir".into());
        }
        let file_tree = get_res!(FileTree::new_from_path(
            self.rep_meta.clone(),
            relative_path
        ));
        self.online_list.push(file_tree);
        Ok(())
    }

    pub fn clear() {
        todo!();
    }
}
