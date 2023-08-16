use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use inotify::{EventMask, WatchDescriptor};
use tokio::sync::RwLock;
use tonic::Request;

use crate::{
    config::RpcChannel,
    conflicts::conflicts_resolve,
    info,
    replica::{
        meta::{delete_file, sync_bytes},
        Meta,
    },
    reptra::{QueryReq, QueryRes, RsyncClient},
    timestamp::{SingletonTime, VectorTime},
    unwrap_res, MyResult,
};

use super::path_local::PathLocal;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NodeStatus {
    Exist,
    Deleted,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModType {
    Create,
    Delete,
    Modify,
    MovedTo,
    MovedFrom,
}

#[derive(Clone)]
pub struct NodeData {
    pub children: HashMap<String, Arc<Node>>,
    pub mod_time: VectorTime,
    pub sync_time: VectorTime,
    pub create_time: SingletonTime,
    pub status: NodeStatus,
    pub wd: Option<WatchDescriptor>,
}

pub struct Node {
    pub meta: Arc<Meta>,
    pub path: Box<PathLocal>,
    pub data: RwLock<NodeData>,
}

#[derive(Clone)]
pub struct SyncOption {
    pub time: i32,
    pub is_dir: bool,
    pub client: RsyncClient<RpcChannel>,
}

#[derive(Clone)]
pub struct ModOption {
    pub ty: ModType,
    pub time: i32,
    pub name: String,
    pub is_dir: bool,
}

impl ModType {
    pub fn from_mask(mask: &EventMask) -> Self {
        if mask.contains(EventMask::CREATE) {
            return ModType::Create;
        } else if mask.contains(EventMask::DELETE) {
            return ModType::Delete;
        } else if mask.contains(EventMask::MODIFY) {
            return ModType::Modify;
        } else if mask.contains(EventMask::MOVED_TO) {
            return ModType::MovedTo;
        } else if mask.contains(EventMask::MOVED_FROM) {
            return ModType::MovedFrom;
        } else {
            panic!("Unknown event mask: {:?}", mask);
        }
    }
}

impl QueryRes {
    pub fn new_from_node(node: &Node, data: &NodeData) -> Self {
        Self {
            deleted: data.status.eq(&NodeStatus::Deleted),
            is_dir: node.path.is_dir(),
            create_id: data.create_time.create_id(),
            create_time: data.create_time.time(),
            mod_time: data.mod_time.clone().into(),
            sync_time: data.sync_time.clone().into(),
            children: data.children.iter().map(|(k, _)| k.clone()).collect(),
        }
    }
}

impl SyncOption {
    pub async fn query_path(&mut self, path: &PathLocal) -> MyResult<QueryRes> {
        Ok(self
            .client
            .query(Request::new(QueryReq {
                path_rel: path.to_rel(),
            }))
            .await
            .or(Err("query failed"))?
            .into_inner())
    }
}

impl Node {
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map_or(format!("replica-{}", self.meta.id), |s| s)
    }

    #[async_recursion]
    pub async fn sub_tree(&self, show_detail: bool, is_last: Vec<bool>) {
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
        if self.path.is_dir() {
            print!("\x1b[1;34m{}\x1b[0m", self.file_name());
        } else {
            print!("{}", self.file_name());
        }

        if show_detail {
            print!(
                "  \x1b[33m{}\x1b[0m",
                self.data.read().await.mod_time.display()
            );
            print!(
                "  \x1b[32m{}\x1b[0m",
                self.data.read().await.sync_time.display()
            );
        }
        println!("");

        let children = &self.data.read().await.children;
        let mut undeleted = Vec::new();

        for (_, child) in children {
            if child.data.read().await.status != NodeStatus::Deleted {
                undeleted.push(child);
            }
        }

        undeleted.sort_by(|a, b| {
            (a.path.is_dir(), a.file_name()).cmp(&(b.path.is_dir(), b.file_name()))
        });

        for child in &undeleted {
            let now_flag = child.file_name() == undeleted.last().unwrap().file_name();
            let mut new_is_last = is_last.clone();
            new_is_last.push(now_flag);
            child.sub_tree(show_detail, new_is_last).await;
        }
    }

    // the replica's bedrock
    pub async fn new_base_node(meta: &Arc<Meta>, path: PathLocal) -> Self {
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::default(),
            sync_time: VectorTime::default(),
            create_time: SingletonTime::default(),
            status: NodeStatus::Exist,
            wd: meta.watch.add_watch(&path).await,
        };
        Self {
            path: Box::new(path),
            meta: meta.clone(),
            data: RwLock::new(data),
        }
    }

    pub async fn new_from_create(path: &PathLocal, time: i32, meta: &Arc<Meta>) -> Self {
        let create_time = SingletonTime::new(meta.id, time);
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::from_singleton_time(&create_time),
            sync_time: VectorTime::from_singleton_time(&create_time),
            create_time,
            status: NodeStatus::Exist,
            wd: meta.watch.add_watch(path).await,
        };
        Node {
            path: Box::new(path.clone()),
            meta: meta.clone(),
            data: RwLock::new(data),
        }
    }

    // the new temporary node which is not exist in the file system
    // when any ground sycnchronization happens, we will make it exist
    pub fn new_tmp(meta: &Arc<Meta>, tmp_path: &PathLocal, sync_time: &VectorTime) -> Self {
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::default(),
            sync_time: sync_time.clone(),
            create_time: SingletonTime::new(0, 0),
            status: NodeStatus::Deleted,
            wd: None,
        };
        Self {
            meta: meta.clone(),
            path: Box::new(tmp_path.clone()),
            data: RwLock::new(data),
        }
    }

    pub fn get_child(&self, data: &NodeData, name: &String) -> Arc<Node> {
        if let Some(child) = data.children.get(name) {
            child.clone()
        } else {
            Arc::new(Node::new_tmp(
                &self.meta,
                &self.path.join_name(name),
                &data.sync_time,
            ))
        }
    }
}

// recursive operation on node and node's data
impl Node {
    // scan all the files (which are not detected before) in the directory
    #[async_recursion]
    pub async fn scan_all(&self, init_time: i32) -> MyResult<()> {
        let mut sub_files = unwrap_res!(tokio::fs::read_dir(self.path.as_ref())
            .await
            .or(Err("Scan All Error : read dir error")));
        while let Some(sub_file) = sub_files.next_entry().await.unwrap() {
            let path = PathLocal::new_from_local(self.path.prefix(), sub_file.path());
            let child = Arc::new(Node::new_from_create(&path, init_time, &self.meta).await);
            if child.path.is_dir() {
                child.scan_all(init_time).await?;
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
    pub async fn handle_modify(&self, mut walk: Vec<String>, op: ModOption) -> MyResult<()> {
        if !walk.is_empty() {
            // not the target node yet
            let mut cur_data = self.data.write().await;
            let child_name = walk.pop().unwrap();
            let child = cur_data
                .children
                .get(&child_name)
                .ok_or("Event Handling Error : Node not found along the path")?;
            child.handle_modify(walk, op.clone()).await?;
            cur_data.mod_time.update_one(self.meta.id, op.time);
        } else {
            match op.ty {
                ModType::Create | ModType::MovedTo => {
                    // create method : from parent node handling it
                    self.create_child(&op.name, op.time).await?;
                }
                ModType::Delete => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found when handling Delete Event")?;
                    deleted.delete_rm(op.time).await?;
                    data.mod_time.update_one(self.meta.id, op.time);
                }
                ModType::Modify => {
                    let mut data = self.data.write().await;
                    let modified = data
                        .children
                        .get(&op.name)
                        .ok_or("Modify Error : Node not found when handling Modify event")?;
                    modified.modify_self(op.time).await?;
                    data.mod_time.update_one(self.meta.id, op.time);
                }
                ModType::MovedFrom => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found when handling MovedTo event")?;
                    deleted.delete_moved_from(op.time).await?;
                    data.mod_time.update_one(self.meta.id, op.time);
                }
            };
        };
        Ok(())
    }

    #[async_recursion]
    pub async fn handle_query(&self, mut walk: Vec<String>) -> MyResult<QueryRes> {
        let cur_data = self.data.read().await;

        // deleted : return directly
        if cur_data.status == NodeStatus::Deleted {
            return Ok(QueryRes::new_from_node(&self, &cur_data));
        }

        if let Some(name) = walk.pop() {
            let child = self.get_child(&cur_data, &name);
            return child.handle_query(walk).await;
        } else {
            return Ok(QueryRes::new_from_node(&self, &cur_data));
        }
    }

    // always sync remote -> local
    #[async_recursion]
    pub async fn handle_sync(
        &self,
        op: SyncOption,
        mut walk: Vec<String>,
        parent_wd: Option<WatchDescriptor>,
    ) -> MyResult<bool> {
        if !walk.is_empty() {
            // not the target node yet
            let mut cur_data = self.data.write().await;
            let name = walk.pop().unwrap();
            let child = self.get_child(&cur_data, &name);
            let sync_flag = child
                .handle_sync(op, walk, cur_data.wd.clone().or(parent_wd))
                .await?;
            let child_data = child.data.read().await.clone();
            cur_data.mod_time.update(&child_data.mod_time);
            if child_data.status == NodeStatus::Exist {
                // may be the node is tmp node
                cur_data.children.insert(name, child);
                cur_data.status = NodeStatus::Exist;
            }
            if cur_data.wd.is_none() {
                // the current node is tmp node
                cur_data.wd = self.meta.watch.add_watch(&self.path).await;
            }
            Ok(sync_flag)
        } else {
            // find the target file(dir) to be synchronized
            if op.is_dir {
                self.sync_dir(op.clone(), &parent_wd.expect("no wd found"))
                    .await
            } else {
                self.sync_file(op.clone(), &parent_wd.expect("no wd found"))
                    .await
            }
        }
    }
}

impl Node {
    pub async fn create_child(&self, name: &String, time: i32) -> MyResult<()> {
        info!(
            "New file created : {} under the dir {}",
            name,
            self.path.display()
        );
        let child_path = self.path.join_name(name);
        let child = Arc::new(Node::new_from_create(&child_path, time, &self.meta).await);
        if child.path.is_dir() {
            let res = child.scan_all(time).await;
            unwrap_res!(res);
        }
        let mut parent_data = self.data.write().await;
        parent_data.children.insert(name.clone(), child);
        parent_data.mod_time.update_one(self.meta.id, time);
        Ok(())
    }

    pub async fn modify_self(&self, time: i32) -> MyResult<()> {
        info!("File modified : {}", self.path.display());
        let mut data = self.data.write().await;
        data.mod_time.update_one(self.meta.id, time);
        data.sync_time.update_one(self.meta.id, time);
        Ok(())
    }

    // just one file, actually removed in file system
    // and the watch descriptor is automatically removed
    pub async fn delete_rm(&self, time: i32) -> MyResult<()> {
        info!("File deleted : {}", self.path.display());
        let mut data = self.data.write().await;
        // the file system cannot delete a directory that is not empty
        for (_, child) in data.children.iter() {
            if child.data.read().await.status == NodeStatus::Exist {
                return Err("Delete Error : Directory not empty".into());
            }
        }
        // clear the mod time && update the sync time
        data.mod_time.clear();
        data.sync_time.update_one(self.meta.id, time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            if let Ok(_) = self.meta.watch.remove_watch(self.path.as_ref(), &wd).await {
                return Err(
                    "Delet Error : Watcher is not automatically removed when \"rm\" a file".into(),
                );
            }
        }
        Ok(())
    }

    // we should manually remove the watcher descriptor here
    // as the file is moved to another space
    #[async_recursion]
    pub async fn delete_moved_from(&self, time: i32) -> MyResult<()> {
        let mut data = self.data.write().await;
        if data.status == NodeStatus::Deleted {
            return Ok(());
        }
        for (_, child) in data.children.iter() {
            unwrap_res!(child.delete_moved_from(time).await);
        }
        info!("File deleted : {}", self.path.display());
        data.mod_time.clear();
        data.sync_time.update_one(self.meta.id, time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            self.meta
                .watch
                .remove_watch(self.path.as_ref(), &wd)
                .await?;
        }
        Ok(())
    }

    // sync a remote folder -> local folder
    #[async_recursion]
    pub async fn sync_dir(&self, mut op: SyncOption, wd: &WatchDescriptor) -> MyResult<bool> {
        let cur_data = self.data.write().await;
        let remote = op.query_path(&self.path).await?;
        if (remote.deleted && cur_data.status == NodeStatus::Deleted)
            || (!remote.deleted && cur_data.sync_time.geq(&remote.mod_time))
        {
            info!("Both deleted, skip the whole dir : {}", self.path.display());
            return Ok(false);
        } else {
            let remote = op.query_path(&self.path).await?;
            let mut name_list: Vec<String> =
                cur_data.children.iter().map(|(k, _)| k.clone()).collect();
            name_list.append(&mut remote.children.clone());
            name_list.sort();
            name_list.dedup();
            let mut join_set = tokio::task::JoinSet::new();
            for name in name_list {
                let child = self.get_child(&cur_data, &name);
                let remote_child = op.query_path(&child.path).await?;
                if child.data.read().await.status == NodeStatus::Exist
                    && !remote_child.deleted
                    && child.path.is_dir() != remote_child.is_dir
                {
                    info!(
                        "Both exist, but one is dir, the other is file : {}",
                        child.path.display()
                    );
                    conflicts_resolve();
                    return Ok(true);
                }
                let child_is_dir = if remote_child.deleted {
                    child.path.is_dir()
                } else {
                    remote_child.is_dir
                };
                if child_is_dir {
                    let op = op.clone();
                    let wd = wd.clone();
                    join_set.spawn(async move {
                        child.sync_dir(op, &wd).await;
                        child.clone()
                    });
                } else {
                    let op = op.clone();
                    let wd = wd.clone();
                    join_set.spawn(async move {
                        child.sync_file(op, &wd).await;
                        child.clone()
                    });
                }
            }
            let mut sync_flag: bool = false;
            while let Some(res) = join_set.join_next().await {
                // sync_flag |=
                // res.or::<String>(Err("Sync Error : Sync child join error".into()))??;
            }
            if remote.deleted && cur_data.mod_time.leq(&remote.sync_time) {
                // Ok, we can delete the local dir
            }
            if cur_data.status == NodeStatus::Deleted && sync_flag {
                // the current local dir is deleted, but there are still changes happened
                // which means that some files are created in the local dir
                // we can create a node for the local dir
            }
            Ok(sync_flag)
        }
    }

    // sync a single remove file to local
    pub async fn sync_file(&self, mut op: SyncOption, wd: &WatchDescriptor) -> MyResult<bool> {
        let data = self.data.read().await.clone();
        // the rw_lock is not required here
        let remote = op.query_path(&self.path).await?;
        if data.status == NodeStatus::Exist && !remote.deleted {
            // both exist
            if data.mod_time.leq(&remote.sync_time) {
                // local_m <= remote_s
                info!("Both exist : override the local file");
                self.sync_override_file(op, &remote, &wd).await?;
                return Ok(true);
            } else if data.sync_time.geq(&remote.mod_time) {
                // local_s >= remote_m
                info!("Both exist : do nothing");
                return Ok(false);
            } else {
                // report conflicts
                info!("Both exist : diverged, conflicts happened");
                conflicts_resolve();
                return Ok(true);
            }
        } else if data.status == NodeStatus::Exist || remote.deleted == false {
            if remote.deleted {
                // remote(deleted) -> local
                if data.create_time.leq_vec(&remote.sync_time) {
                    if data.mod_time.leq(&remote.sync_time) {
                        info!("Sync from deleted : delete the local file");
                        self.sync_delete_file(op, &remote, &wd).await?;
                        return Ok(true);
                    } else {
                        info!("Sync from deleted : but changes diverged, conflicts happened");
                        conflicts_resolve();
                        return Ok(true);
                    }
                } else {
                    info!("Sync from deleted : independent files, do nothing");
                    return Ok(false);
                }
            } else {
                // remote -> local(deleted)
                let (id, time) = (remote.create_id, remote.create_time);
                if data.sync_time.geq_singleton(id, time) {
                    if data.sync_time.geq(&remote.mod_time) {
                        info!("Sync to deleted : do nothing");
                        return Ok(false);
                    } else {
                        info!("Sync to deleted : but changes diverged, conflicts happened");
                        conflicts_resolve();
                        return Ok(true);
                    }
                } else {
                    info!("Sync to deleted : independent files, create a new copy");
                    self.sync_create_file(op, &remote, &wd).await?;
                    return Ok(true);
                }
            }
        } else {
            info!("Neither exists : do nothing");
            return Ok(false);
        }
    }

    pub async fn sync_override_file(
        &self,
        op: SyncOption,
        remote: &QueryRes,
        wd: &WatchDescriptor,
    ) -> MyResult<()> {
        let mut cur_data = self.data.write().await;
        assert!(cur_data.wd.is_none());
        self.meta.watch.freeze_watch(wd).await;
        sync_bytes(&self.path, op.client).await?;
        self.meta.watch.unfreeze_watch(wd).await;
        cur_data.mod_time.update(&remote.mod_time.clone().into());
        cur_data.sync_time.update(&remote.sync_time.clone().into());
        cur_data.sync_time.update_one(self.meta.id, op.time);
        Ok(())
    }

    pub async fn sync_create_file(
        &self,
        op: SyncOption,
        remote: &QueryRes,
        wd: &WatchDescriptor,
    ) -> MyResult<()> {
        let mut cur_data = self.data.write().await;
        assert!(cur_data.wd.is_none());
        self.meta.watch.freeze_watch(wd).await;
        sync_bytes(&self.path, op.client).await?;
        self.meta.watch.unfreeze_watch(wd).await;
        cur_data.mod_time = remote.mod_time.clone().into();
        cur_data.sync_time = remote.sync_time.clone().into();
        cur_data.sync_time.update_one(self.meta.id, op.time);
        cur_data.create_time = SingletonTime::new(remote.create_id, remote.create_time);
        cur_data.status = NodeStatus::Exist;
        // only a folder can have a watch descriptor
        Ok(())
    }

    pub async fn sync_delete_file(
        &self,
        op: SyncOption,
        remote: &QueryRes,
        wd: &WatchDescriptor,
    ) -> MyResult<()> {
        let mut cur_data = self.data.write().await;
        assert!(cur_data.wd.is_none());
        self.meta.watch.freeze_watch(wd).await;
        delete_file(&self.path).await?;
        self.meta.watch.unfreeze_watch(wd).await;
        cur_data.mod_time.clear();
        cur_data.sync_time.update(&remote.sync_time.clone().into());
        cur_data.sync_time.update_one(self.meta.id, op.time);
        cur_data.status = NodeStatus::Deleted;
        Ok(())
    }
}
