use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_recursion::async_recursion;

use inotify::WatchDescriptor;
use tokio::sync::RwLock;

use crate::{replica::RepMeta, unwrap_res, MyResult};

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
    pub(crate) wd: Option<WatchDescriptor>,
}

pub struct Node {
    pub rep_meta: Arc<RepMeta>,
    pub path: Box<PathBuf>,
    pub is_dir: bool,
    pub data: RwLock<NodeData>,
    pub parent: Option<Weak<Node>>,
}

// basic methods
impl Node {
    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    // the replica's bedrock
    pub fn new_trees_collect(rep_meta: Arc<RepMeta>, file_watcher: &mut FileWatcher) -> Self {
        let path = rep_meta.prefix.clone();
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::default(),
            sync_time: VectorTime::default(),
            create_time: SingletonTime::default(),
            status: NodeStatus::Exist,
            wd: file_watcher.add_watch(&path),
        };
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
        let create_time = SingletonTime::new(rep_meta.id, time);
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::from_singleton_time(&create_time),
            sync_time: VectorTime::from_singleton_time(&create_time),
            create_time,
            status: NodeStatus::Exist,
            wd: file_watcher.add_watch(path),
        };
        Node {
            path: Box::new(path.clone()),
            is_dir: rep_meta.check_is_dir(path),
            rep_meta,
            data: RwLock::new(data),
            parent,
        }
    }
}

// direct operation on node and node's data
impl Node {
    pub async fn create(
        &self,
        name: &String,
        time: usize,
        parent: Weak<Node>,
        file_watcher: &mut FileWatcher,
    ) -> MyResult<()> {
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
        let mut parent_data = self.data.write().await;
        parent_data.children.insert(name.clone(), child);
        parent_data
            .mod_time
            .update_singleton(self.rep_meta.id, time);
        Ok(())
    }

    pub async fn modify(&self, id: u16, time: usize) {
        let mut data = self.data.write().await;
        data.mod_time.update_singleton(id, time);
        data.sync_time.update_singleton(id, time);
    }

    // just one file, actually removed in file system
    // and the watch descriptor is automatically removed
    pub async fn delete_rm(
        &self,
        id: u16,
        time: usize,
        file_watcher: &mut FileWatcher,
    ) -> MyResult<()> {
        let mut data = self.data.write().await;
        // the file system cannot delete a directory that is not empty
        for (_, child) in data.children.iter() {
            if child.data.read().await.status == NodeStatus::Exist {
                return Err("Delete Error : Directory not empty".into());
            }
        }
        // clear the mod time && update the sync time
        data.mod_time.clear();
        data.sync_time.update_singleton(id, time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            if let Ok(_) = file_watcher.remove_watch(&self.path, &wd) {
                return Err(
                    "Delet Error : Watcher is not automatically removed when \"rm\" a file".into(),
                );
            }
        } else if self.is_dir {
            return Err("Delete Error : No Watch Descriptor".into());
        }
        Ok(())
    }

    // we should manually remove the watcher descriptor here
    // as the file is moved to another space
    #[async_recursion]
    pub async fn delete_moved_from(
        &self,
        id: u16,
        time: usize,
        file_watcher: &mut FileWatcher,
    ) -> MyResult<()> {
        let mut data = self.data.write().await;
        if data.status == NodeStatus::Deleted {
            return Ok(());
        }
        for (_, child) in data.children.iter() {
            unwrap_res!(child.delete_moved_from(id, time, file_watcher).await);
        }
        data.mod_time.clear();
        data.sync_time.update_singleton(id, time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            file_watcher.remove_watch(&self.path, &wd)?;
        } else if self.is_dir {
            return Err("Delete Error : No Watch Descriptor".into());
        }
        Ok(())
    }
}

impl Node {
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
    ) -> MyResult<()> {
        // not the target node yet
        if !walk.is_empty() {
            let mut cur_data = self.data.write().await;
            let child_name = walk.pop().unwrap();
            let child = cur_data
                .children
                .get(&child_name)
                .ok_or("Event Handling Error : Node not found along the path")?;
            let child_weak = Arc::downgrade(child);
            child
                .handle_event(path, walk, op.clone(), child_weak, file_watcher)
                .await?;
            cur_data
                .mod_time
                .update_singleton(self.rep_meta.id, op.time);
        } else {
            match op.ty {
                ModType::Create | ModType::MovedTo => {
                    self.create(&op.name, op.time, cur_weak, file_watcher)
                        .await?;
                }
                ModType::Delete => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found")?;
                    deleted
                        .delete_rm(self.rep_meta.id, op.time, file_watcher)
                        .await?;
                    data.mod_time.update_singleton(self.rep_meta.id, op.time);
                }
                ModType::Modify => {
                    self.modify(self.rep_meta.id, op.time).await;
                }
                ModType::MovedFrom => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found when handling MovedTo event")?;
                    deleted
                        .delete_moved_from(self.rep_meta.id, op.time, file_watcher)
                        .await?;
                    data.mod_time.update_singleton(self.rep_meta.id, op.time);
                }
            };
        };
        Ok(())
    }
}
