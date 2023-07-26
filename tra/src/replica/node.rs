use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;

use tokio::sync::RwLock;

use crate::{get_res, replica::RepMeta, MyResult};

use super::{
    timestamp::{CreateTime, VectorTime},
    ModOption, ModType,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NodeStatus {
    Exist,
    Deleted,
    Unknow,
}

pub struct NodeData {
    pub children: HashMap<String, Arc<Node>>,
    pub mod_time: VectorTime,
    pub sync_time: VectorTime,
    pub create_time: CreateTime,
    pub status: NodeStatus,
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
    pub fn new_from_create(
        path: &PathBuf,
        create_time: CreateTime,
        rep_meta: Arc<RepMeta>,
    ) -> Self {
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::from_create_time(&create_time),
            sync_time: VectorTime::from_create_time(&create_time),
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

    // sync from another replica, so create a file
    pub fn new_from_sync(path: &PathBuf, rep_meta: Arc<RepMeta>) -> Self {
        todo!()
    }

    #[async_recursion]
    pub async fn init_subfiles(&mut self, init_time: &CreateTime) -> MyResult<()> {
        let static_path = self.path.as_path();
        let mut sub_files = get_res!(tokio::fs::read_dir(static_path).await);
        while let Some(sub_file) = get_res!(sub_files.next_entry().await) {
            let mut new_node =
                Node::new_from_create(&sub_file.path(), init_time.clone(), self.rep_meta.clone());
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
    pub async fn modify(
        &self,
        path: &PathBuf,
        mut walk: Vec<String>,
        op: ModOption,
    ) -> MyResult<()> {
        let mut data = self.data.write().await;

        if walk.len() == 1 && op.create_time().is_some() {
            // create a new file in its parent dir
            if data.children.contains_key(&walk[0]) {
                return Err("Modify Error : node already exists when creating".into());
            }
            let create_time = op.create_time().unwrap();
            let new_node = Node::new_from_create(path, create_time, self.rep_meta.clone());
            data.children
                .insert(new_node.file_name(), Arc::new(new_node));
        } else if walk.len() != 0 {
            // not in this level
            let name = walk.pop().unwrap();
            if let Some(next_node) = data.children.get(&name) {
                next_node.modify(path, walk, op).await?;
            } else {
                return Err("Modify Error : node not exists when modifying".into());
            }
        } else {
            // modify file
            if op.ty == ModType::Modify {
                todo!()
            } else if op.ty == ModType::Delete {
                todo!()
            } else {
                return Err("Modify Error : not supposed to be create here".into());
            }
        }
        Ok(())
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
