use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;

use tokio::sync::RwLock;

use crate::{get_res, replica::RepMeta, MyResult};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NodeStatus {
    Exist,
    Deleted,
    Unknow,
}

pub struct NodeData {
    pub children: HashMap<String, Arc<Node>>,
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

    // only new when creating or syncing a new file
    pub fn new(path: &PathBuf, rep_meta: Arc<RepMeta>) -> Self {
        let data = NodeData {
            children: HashMap::new(),
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
    pub async fn init_subfiles(&mut self) -> MyResult<()> {
        let static_path = self.path.as_path();
        let mut sub_files = get_res!(tokio::fs::read_dir(static_path).await);
        while let Some(sub_file) = get_res!(sub_files.next_entry().await) {
            let mut new_node = Node::new(&sub_file.path(), self.rep_meta.clone());
            if new_node.is_dir {
                new_node.init_subfiles().await?;
            }
            self.data
                .write()
                .await
                .children
                .insert(new_node.file_name(), Arc::new(new_node));
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
