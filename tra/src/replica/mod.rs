pub mod checker;
pub mod file_sync;
pub mod file_watcher;
pub mod node;
pub mod timestamp;

use std::{path::PathBuf, sync::Arc};

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
    pub trees_collect: Arc<Node>,
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
        let rep_meta = Arc::new(RepMeta::new(port));
        Self {
            rep_meta: rep_meta.clone(),
            trees_collect: Arc::new(Node::new_trees_collect(rep_meta)),
        }
    }

    pub async fn init_file_trees(&mut self) -> MyResult<()> {
        // init the whole file tree, all inintial is in time 1
        let init_counter = self.rep_meta.add_counter().await;
        get_res!(self.trees_collect.init_subfiles(init_counter).await);
        Ok(())
    }

    // modify && create && delete
    pub async fn modify(&self, path: &PathBuf, op: ModOption) -> MyResult<()> {
        let walk = self.rep_meta.decompose(path);

        // do not need to update the modify time
        // but use get_res! to check the result
        get_res!(self.trees_collect.modify(path, walk, op).await);

        Ok(())
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

        let trees_data = self.trees_collect.data.read().await;

        let mut it = trees_data.children.iter();

        while let Some((name, node)) = it.next() {
            tmp_list.push((node.is_dir, name));
        }

        tmp_list.sort_by(|a, b| a.cmp(b));

        for (_, name) in &tmp_list {
            let now_flag = *name == tmp_list.last().unwrap().1;
            let new_is_last = vec![now_flag];
            trees_data
                .children
                .get(*name)
                .unwrap()
                .tree(new_is_last)
                .await;
        }
    }
}
