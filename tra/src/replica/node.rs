use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_recursion::async_recursion;

use tokio::sync::RwLock;

use crate::{debug, error, replica::RepMeta, unwrap_res, MyResult};

use super::{
    file_watcher::FileWatcher,
    timestamp::{SingletonTime, VectorTime},
    ModOption, ModType,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NodeStatus {
    Exist,
    Deleted,
}

pub struct NodeData {
    pub(crate) children: HashMap<String, Arc<Node>>,
    pub(crate) mod_time: VectorTime,
    pub(crate) sync_time: VectorTime,
    pub(crate) create_time: SingletonTime,
    pub(crate) status: NodeStatus,
}

impl NodeData {
    pub fn modify(&mut self, id: u16, time: usize) {
        self.mod_time.update(id, time);
        self.sync_time.update(id, time);
    }

    pub fn delete(&mut self, id: u16, time: usize) -> VectorTime {
        // the file system cannot delete a directory that is not empty
        // TODO : assert!(self.children.is_empty());

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
    pub parent: Option<Weak<Node>>,
}

impl Node {
    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    // the replica's bedrock
    pub fn new_trees_collect(rep_meta: Arc<RepMeta>, file_watcher: &mut FileWatcher) -> Self {
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::default(),
            sync_time: VectorTime::default(),
            create_time: SingletonTime::default(),
            status: NodeStatus::Exist,
        };
        let path = rep_meta.prefix.clone();
        file_watcher.add_watch(&path);
        Self {
            path: Box::new(path),
            rep_meta,
            is_dir: true,
            data: RwLock::new(data),
            parent: None,
        }
    }

    // locally create a file
    pub fn new_from_create(
        path: &PathBuf,
        time: usize,
        rep_meta: Arc<RepMeta>,
        parent: Option<Weak<Node>>,
        file_watcher: &mut FileWatcher,
    ) -> Self {
        let create_time = SingletonTime::new(rep_meta.port, time);
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::from_singleton_time(&create_time),
            sync_time: VectorTime::from_singleton_time(&create_time),
            create_time,
            status: NodeStatus::Exist,
        };
        let is_dir = rep_meta.check_is_dir(path);
        file_watcher.add_watch(path);
        Node {
            rep_meta,
            path: Box::new(path.clone()),
            is_dir,
            data: RwLock::new(data),
            parent,
        }
    }

    // scan all the files (which are not detected before) in the directory
    #[async_recursion]
    pub async fn scan_all(
        &self,
        init_time: usize,
        cur_weak: Weak<Node>,
        file_watcher: &mut FileWatcher,
    ) -> MyResult<()> {
        let static_path = self.path.as_path();
        let mut sub_files = unwrap_res!(tokio::fs::read_dir(static_path)
            .await
            .or(Err("Scan All Error : read dir error")));
        while let Some(sub_file) = sub_files.next_entry().await.unwrap() {
            let child = Arc::new(Node::new_from_create(
                &sub_file.path(),
                init_time,
                self.rep_meta.clone(),
                Some(cur_weak.clone()),
                file_watcher,
            ));
            if child.is_dir {
                child
                    .scan_all(init_time, Arc::downgrade(&child), file_watcher)
                    .await?;
            }
            self.data
                .write()
                .await
                .children
                .insert(child.file_name(), child);
        }
        Ok(())
    }

    #[async_recursion]
    pub async fn handle_event(
        &self,
        path: &PathBuf,
        mut walk: Vec<String>,
        op: ModOption,
        cur_weak: Weak<Node>,
        file_watcher: &mut FileWatcher,
    ) -> MyResult<VectorTime> {
        // not the target node yet
        if !walk.is_empty() {
            let mut cur_data = self.data.write().await;
            let child_name = walk.pop().unwrap();
            let child = cur_data
                .children
                .get(&child_name)
                .ok_or("Event Handling Error : Node not found along the path")?;
            let child_weak = Arc::downgrade(child);
            let new_mod_time = child
                .handle_event(path, walk, op, child_weak, file_watcher)
                .await?;
            cur_data.mod_time.chkmax(&new_mod_time);
            Ok(new_mod_time)
        } else {
            error!("type : {:?}, name : {}", op.ty, op.name);
            match op.ty {
                ModType::Create | ModType::MovedFrom => {
                    self.handle_create(&op.name, op.time, cur_weak, file_watcher)
                        .await
                }
                ModType::Delete => Ok(self.handle_delete(&op.name, op.time).await),
                ModType::Modify => Ok(self.handle_modify(op.time).await),
                ModType::MovedTo => {
                    self.handle_moved_to(&op.name, op.time, cur_weak).await;
                    todo!();
                }
            }
        }
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
        let mut undeleted = Vec::new();

        for (_, child) in children {
            if child.data.read().await.status != NodeStatus::Deleted {
                undeleted.push(child);
            }
        }

        undeleted.sort_by(|a, b| (a.is_dir, a.file_name()).cmp(&(b.is_dir, b.file_name())));

        for child in &undeleted {
            let now_flag = child.file_name() == undeleted.last().unwrap().file_name();
            let mut new_is_last = is_last.clone();
            new_is_last.push(now_flag);
            child.tree(new_is_last).await;
        }
    }
}

// direct handle on the node (do not need to recursively get the node)
impl Node {
    pub async fn pushup_mtime(&self, mod_time: &VectorTime) {
        // update the mod time
        self.data.write().await.mod_time.chkmax(&mod_time);
        let mut ancestor = self.parent.clone();
        while ancestor.is_some() {
            let upgraded = ancestor.unwrap().upgrade().unwrap();
            upgraded.data.write().await.mod_time.chkmax(&mod_time);
            ancestor = upgraded.parent.clone();
        }
    }

    pub async fn handle_create(
        &self,
        name: &String,
        time: usize,
        parent: Weak<Node>,
        file_watcher: &mut FileWatcher,
    ) -> MyResult<VectorTime> {
        let child_path = self.path.join(name);
        let child = Arc::new(Node::new_from_create(
            &child_path,
            time,
            self.rep_meta.clone(),
            Some(parent),
            file_watcher,
        ));
        if child.is_dir {
            let res = child
                .scan_all(time, Arc::downgrade(&child), file_watcher)
                .await;
            unwrap_res!(res);
        }
        let mod_time = child.data.read().await.mod_time.clone();
        let mut parent_data = self.data.write().await;
        parent_data.children.insert(name.clone(), child);
        parent_data.mod_time.chkmax(&mod_time);
        Ok(mod_time)
    }

    pub async fn handle_delete(&self, name: &String, time: usize) -> VectorTime {
        let mut cur_data = self.data.write().await;
        let to_be_deleted = cur_data.children.get(name).unwrap().clone();
        let mod_time = to_be_deleted
            .data
            .write()
            .await
            .delete(self.rep_meta.port, time);
        cur_data.mod_time.chkmax(&mod_time);
        mod_time
    }

    pub async fn handle_modify(&self, time: usize) -> VectorTime {
        let mut cur_data = self.data.write().await;
        cur_data.modify(self.rep_meta.port, time);
        cur_data.mod_time.clone()
    }

    pub async fn handle_moved_to(&self, name: &String, time: usize, parent: Weak<Node>) {
        // self.handle_create(name, time, parent).await;
        // let child = self.data.read().await.children.get(name).unwrap().clone();
        // child.scan_all(time, Arc::downgrade(&child)).await.unwrap();
    }
}
