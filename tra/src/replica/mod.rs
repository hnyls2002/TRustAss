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

    pub fn decompose(&self, path: &PathBuf) -> Vec<String> {
        let mut tmp_path = path.clone();
        let mut ret: Vec<String> = Vec::new();
        while tmp_path.file_name().is_some() {
            if tmp_path == self.prefix {
                break;
            }
            ret.push(tmp_path.file_name().unwrap().to_str().unwrap().to_string());
            tmp_path.pop();
        }
        ret
    }

    pub async fn read_counter(&self) -> usize {
        self.counter.read().await.clone()
    }

    pub async fn add_counter(&self) -> usize {
        let mut now = self.counter.write().await;
        *now += 1;
        *now
    }
}

pub struct Replica {
    pub rep_meta: Arc<RepMeta>,
    pub file_trees: RwLock<HashMap<String, Arc<Node>>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModType {
    Modify,
    Create,
    Delete,
}

#[derive(Copy, Clone)]
pub struct ModOption {
    pub ty: ModType,
    pub time: usize,
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
        // init the whole file tree, all inintial is in time 1
        let init_counter = self.rep_meta.add_counter().await;

        let mut tree_list = get_res!(tokio::fs::read_dir(&self.rep_meta.prefix).await);
        let mut file_trees = self.file_trees.write().await;
        while let Some(tree_root) = get_res!(tree_list.next_entry().await) {
            let path = tree_root.path();
            let mut new_node = Node::new_from_create(&path, init_counter, self.rep_meta.clone());
            if new_node.is_dir {
                new_node.init_subfiles(init_counter).await?;
            }
            file_trees.insert(new_node.file_name(), Arc::new(new_node));
        }
        Ok(())
    }

    // modify && create && delete
    pub async fn modify(&self, path: &PathBuf, op: ModOption) -> MyResult<()> {
        let mut walk = self.rep_meta.decompose(path);

        if walk.len() == 0 {
            // empty path
            return Err("Modify Error : empty path".into());
        } else if walk.len() == 1 && op.ty == ModType::Create {
            // detect creating a new file in the root dir
            let mut file_trees = self.file_trees.write().await;
            if file_trees.contains_key(&walk[0]) {
                return Err("Modify Error : node already exists when creating".into());
            }
            let new_node = Node::new_from_create(path, op.time, self.rep_meta.clone());
            file_trees.insert(new_node.file_name(), Arc::new(new_node));
        } else {
            // other cases
            let root_name = walk.pop().unwrap();
            let file_trees = self.file_trees.read().await;
            if let Some(root_node) = file_trees.get(&root_name) {
                root_node.modify(path, walk, op).await?;
            } else {
                return Err("Modify Error : root node not found".into());
            }
        }

        // update timestamp
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
