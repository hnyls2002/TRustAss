use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;

use tokio::sync::RwLock;

use crate::{get_res, replica::RepMeta, MyResult};

use super::{
    timestamp::{SingletonTime, VectorTime},
    ModOption, ModType,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NodeStatus {
    Exist,
    Deleted,
}

pub struct NodeData {
    pub children: HashMap<String, Arc<Node>>,
    pub mod_time: VectorTime,
    pub sync_time: VectorTime,
    pub create_time: SingletonTime,
    pub status: NodeStatus,
}

impl NodeData {
    pub fn modify(&mut self, id: u16, time: usize) {
        self.mod_time.update(id, time);
        self.sync_time.update(id, time);
    }

    pub fn delete(&mut self, id: u16, time: usize) -> VectorTime {
        // the file system cannot delete a directory that is not empty
        assert!(self.children.is_empty());

        // a file once deleted, the mod time is useless
        // however, its ancestor may still need it
        let mut ret = self.mod_time.clone();
        ret.update(id, time);
        self.mod_time.clear();

        self.sync_time.update(id, time);
        // not clear create_time here, though it is useless
        self.status = NodeStatus::Deleted;

        ret
    }
}

pub struct Node {
    pub rep_meta: Arc<RepMeta>,
    pub path: Box<PathBuf>,
    pub is_dir: bool,
    pub data: RwLock<NodeData>,
}

impl Node {
    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    // locally create a file
    pub fn new_from_create(path: &PathBuf, time: usize, rep_meta: Arc<RepMeta>) -> Self {
        let create_time = SingletonTime::new(rep_meta.port, time);
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::from_singleton_time(&create_time),
            sync_time: VectorTime::from_singleton_time(&create_time),
            create_time,
            status: NodeStatus::Exist,
        };
        let is_dir = rep_meta.check_is_dir(path);
        Node {
            rep_meta,
            path: Box::new(path.clone()),
            is_dir,
            data: RwLock::new(data),
        }
    }

    #[async_recursion]
    pub async fn init_subfiles(&mut self, init_time: usize) -> MyResult<()> {
        let static_path = self.path.as_path();
        let mut sub_files = get_res!(tokio::fs::read_dir(static_path).await);
        while let Some(sub_file) = get_res!(sub_files.next_entry().await) {
            let mut new_node =
                Node::new_from_create(&sub_file.path(), init_time, self.rep_meta.clone());
            if new_node.is_dir {
                new_node.init_subfiles(init_time).await?;
            }
            self.data
                .write()
                .await
                .children
                .insert(new_node.file_name(), Arc::new(new_node));
        }
        Ok(())
    }

    #[async_recursion]
    // get the child's write lock
    pub async fn modify(
        &self,
        path: &PathBuf,
        mut walk: Vec<String>,
        op: ModOption,
    ) -> MyResult<VectorTime> {
        let mut self_data = self.data.write().await;

        if walk.len() == 1 && op.ty == ModType::Create {
            // create a new file in its parent dir
            if self_data.children.contains_key(&walk[0]) {
                return Err("Modify Error : node already exists when creating".into());
            }
            let new_node = Node::new_from_create(path, op.time, self.rep_meta.clone());
            // update the mod time when creating
            self_data
                .mod_time
                .chkmax(&new_node.data.read().await.mod_time);
            self_data
                .children
                .insert(new_node.file_name(), Arc::new(new_node));
        } else if walk.len() != 0 {
            // not in this level
            let name = walk.pop().unwrap();
            if let Some(next_node) = self_data.children.get(&name) {
                let child_mod_time = next_node.modify(path, walk, op).await?;
                // only update the mod time here
                self_data.mod_time.chkmax(&child_mod_time);
            } else {
                return Err("Modify Error : node not exists when modifying".into());
            }
        } else {
            // modify or delete file here
            if op.ty == ModType::Modify {
                self_data.modify(self.rep_meta.port, op.time);
            } else if op.ty == ModType::Delete {
                // the node with deletion notice would not have mod time
                return Ok(self_data.delete(self.rep_meta.port, op.time));
            } else {
                return Err("Modify Error : not supposed to be create here".into());
            }
        }
        Ok(self_data.mod_time.clone())
    }
}

impl Node {
    #[async_recursion]
    pub async fn tree(&self, is_last: Vec<bool>) {
        // println!("{}", self.path.display());
        for i in 0..is_last.len() {
            let flag = is_last.get(i).unwrap();
            if i != is_last.len() - 1 {
                if *flag {
                    print!("    ");
                } else {
                    print!("│   ");
                }
            } else {
                if *flag {
                    print!("└── ");
                } else {
                    print!("├── ");
                }
            }
        }
        if self.is_dir {
            println!("\x1b[1;34m{}\x1b[0m", self.file_name());
        } else {
            println!("{}", self.file_name());
        }

        let children = &self.data.read().await.children;

        let mut it = children.iter().peekable();

        let mut tmp_list = Vec::new();

        while let Some((name, child)) = it.next() {
            tmp_list.push((child.is_dir, name));
        }

        tmp_list.sort_by(|a, b| a.cmp(b));

        for (_, name) in &tmp_list {
            let now_flag = *name == tmp_list.last().unwrap().1;
            let mut new_is_last = is_last.clone();
            new_is_last.push(now_flag);
            children.get(*name).unwrap().tree(new_is_last).await;
        }
    }
}
