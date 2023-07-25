pub mod checker;
pub mod file_sync;
pub mod file_watcher;
pub mod node;
pub mod timestamp;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{config::TMP_PATH, get_res, MyResult};

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

    pub async fn init_file_trees(&mut self) -> MyResult<()> {
        let mut tree_list = get_res!(tokio::fs::read_dir(&self.rep_meta.prefix).await);
        let mut file_trees = self.file_trees.write().await;
        while let Some(tree_root) = get_res!(tree_list.next_entry().await) {
            let path = tree_root.path();
            let mut new_node = Node::new(&path, self.rep_meta.clone());
            if new_node.is_dir {
                new_node.init_subfiles().await?;
            }
            file_trees.insert(new_node.file_name(), Arc::new(new_node));
        }
        Ok(())
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

impl Replica {
    pub async fn tree(&self) {
        let mut tmp_list = Vec::new();

        let file_trees = self.file_trees.read().await;

        let mut it = file_trees.iter();

        while let Some((name, node)) = it.next() {
            tmp_list.push((node.is_dir, name));
        }

        tmp_list.sort_by(|a, b| a.cmp(b));

        for (_, name) in &tmp_list {
            let now_flag = *name == tmp_list.last().unwrap().1;
            let new_is_last = vec![now_flag];
            file_trees.get(*name).unwrap().tree(new_is_last).await;
        }
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
