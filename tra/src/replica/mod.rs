pub mod checker;
pub mod file_sync;
pub mod file_watcher;
pub mod node;
pub mod timestamp;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{config::TMP_PATH, MyResult};

use self::node::{Node, NodeStatus};

pub struct RepMeta {
    pub port: u16,
    pub prefix: PathBuf,
    pub counter: RwLock<usize>,
}

impl RepMeta {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            prefix: PathBuf::from(format!("{}{}", TMP_PATH, port)),
            counter: RwLock::new(0),
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

    pub async fn read_counter(&self) -> usize {
        self.counter.read().await.clone()
    }

    pub async fn add_counter(&mut self) -> usize {
        let mut now = self.counter.write().await;
        *now += 1;
        *now
    }
}

pub struct Replica {
    pub rep_meta: Arc<RepMeta>,
    pub file_trees: RwLock<HashMap<String, Arc<Node>>>,
}

pub enum ModType {
    Modify,
    Create,
    Delete,
}

pub struct ModOption {
    pub ty: ModType,
    pub is_dir: bool,
}

impl Replica {
    pub fn new(port: u16) -> Self {
        Self {
            rep_meta: Arc::new(RepMeta::new(port)),
            file_trees: RwLock::new(HashMap::new()),
        }
    }

    // modify && create && delete
    pub fn modify(&mut self) -> MyResult<()> {
        todo!()
    }

    pub fn sync_dir(&mut self) -> MyResult<()> {
        todo!()
    }

    pub fn sync_file(&mut self) -> MyResult<()> {
        todo!()
    }

    pub fn clear() {
        todo!();
    }
}

pub fn decompose(path: &PathBuf) -> Vec<String> {
    let mut tmp_path = path.clone();
    let mut ret: Vec<String> = Vec::new();
    while tmp_path.file_name().is_some() {
        ret.push(tmp_path.file_name().unwrap().to_str().unwrap().to_string());
        tmp_path.pop();
    }
    ret
}
