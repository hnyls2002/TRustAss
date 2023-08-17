use std::{collections::HashMap, ops::BitOrAssign, sync::Arc};

use async_recursion::async_recursion;
use inotify::{EventMask, WatchDescriptor};
use tokio::sync::{RwLock, RwLockWriteGuard};
use tonic::Request;

use crate::{
    config::RpcChannel,
    conflicts::conflicts_resolve,
    info,
    replica::{
        meta::{delete_empty_dir, delete_file, sync_bytes},
        Meta,
    },
    reptra::{QueryReq, QueryRes, RsyncClient},
    timestamp::{SingletonTime, VectorTime},
    unwrap_res, MyResult,
};

use super::{path_local::PathLocal, query::RemoteData};

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
    pub client: RsyncClient<RpcChannel>,
}

pub enum SyncType {
    Create,
    Override,
    Delete,
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

impl BitOrAssign for NodeStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        // any exist is exist
        if rhs == NodeStatus::Exist {
            *self = NodeStatus::Exist;
        }
    }
}

impl NodeStatus {
    pub fn deleted(&self) -> bool {
        self == &NodeStatus::Deleted
    }

    pub fn exist(&self) -> bool {
        self == &NodeStatus::Exist
    }

    pub fn set_deleted(&mut self) {
        *self = NodeStatus::Deleted;
    }

    pub fn set_exist(&mut self) {
        *self = NodeStatus::Exist;
    }
}

impl SyncOption {
    pub async fn query_data(&mut self, path: &PathLocal) -> MyResult<(RemoteData, bool)> {
        let res = self
            .client
            .query(Request::new(QueryReq {
                path_rel: path.to_rel(),
            }))
            .await
            .or(Err("query failed"))?
            .into_inner();
        Ok(res.to_data())
    }
}

impl NodeData {
    pub async fn pushup_mod(&mut self) {
        self.mod_time = VectorTime::default();
        for (_, child) in &self.children {
            self.mod_time.check_max(&child.data.read().await.mod_time);
        }
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
            if child.data.read().await.status.exist() {
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
        if cur_data.status.deleted() {
            return Ok(QueryRes::from_data(&cur_data, self.path.is_dir()));
        }

        if let Some(name) = walk.pop() {
            let child = self.get_child(&cur_data, &name);
            return child.handle_query(walk).await;
        } else {
            return Ok(QueryRes::from_data(&cur_data, self.path.is_dir()));
        }
    }

    // always sync remote -> local
    #[async_recursion]
    pub async fn handle_sync(
        &self,
        op: SyncOption,
        mut walk: Vec<String>,
        p_wd: Option<WatchDescriptor>,
    ) -> MyResult<NodeStatus> {
        if !walk.is_empty() {
            // not the target node yet
            let mut cur_data = self.data.write().await;
            let np_wd = cur_data.wd.clone().or(p_wd);
            let child = self.get_child(&cur_data, &walk.pop().unwrap());
            let child_status = child.handle_sync(op, walk, np_wd).await?;

            if child_status.exist() {
                // may be the node is tmp node
                cur_data.children.insert(child.file_name(), child);
                cur_data.status.set_exist();
                if cur_data.wd.is_none() {
                    // the current node is tmp node
                    cur_data.wd = self.meta.watch.add_watch(&self.path).await;
                }
            }
            cur_data.pushup_mod().await;
            return Ok(cur_data.status);
        } else {
            return self.sync_node(op, p_wd).await;
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
            if child.data.read().await.status.exist() {
                return Err("Delete Error : Directory not empty".into());
            }
        }
        data.mod_time.update_one(self.meta.id, time);
        data.sync_time.update_one(self.meta.id, time);
        data.status.set_deleted();
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
        if data.status.deleted() {
            return Ok(());
        }
        for (_, child) in data.children.iter() {
            unwrap_res!(child.delete_moved_from(time).await);
        }
        info!("File deleted : {}", self.path.display());
        data.sync_time.update_one(self.meta.id, time);
        data.sync_time.update_one(self.meta.id, time);
        data.status.set_deleted();
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            self.meta
                .watch
                .remove_watch(self.path.as_ref(), &wd)
                .await?;
        }
        Ok(())
    }

    #[async_recursion]
    pub async fn sync_node(
        &self,
        mut op: SyncOption,
        p_wd: Option<WatchDescriptor>,
    ) -> MyResult<NodeStatus> {
        let mut cur_data = self.data.write().await;
        let (remote_data, remote_is_dir) = op.query_data(&self.path).await?;
        let np_wd = cur_data.wd.clone().or(p_wd.clone());

        if cur_data.status.deleted() && remote_data.status.deleted() {
            info!("Both deleted, skip the file : {}", self.path.display());
            return Ok(cur_data.status);
        }

        if remote_data.status.exist() && remote_data.mod_time.leq(&cur_data.sync_time) {
            info!("Local is newer, skip the file : {}", self.path.display());
            return Ok(cur_data.status);
        }

        if cur_data.status.exist()
            && remote_data.status.exist()
            && self.path.is_dir() != remote_is_dir
        {
            info!("Different type, skip the file : {}", self.path.display());
            return Ok(cur_data.status);
        }

        if (cur_data.status.exist() && !self.path.is_dir())
            || (remote_data.status.exist() && !remote_is_dir)
        {
            return self.sync_file(op, np_wd, &mut cur_data, &remote_data).await;
        }

        // sync a remote folder -> local folder

        let mut name_list: Vec<String> = cur_data.children.iter().map(|(k, _)| k.clone()).collect();
        name_list.append(&mut remote_data.children.clone());
        name_list.sort();
        name_list.dedup();

        let mut join_set = tokio::task::JoinSet::new();

        for name in name_list {
            let child = self.get_child(&cur_data, &name);
            let op = op.clone();
            let np_wd = np_wd.clone();
            join_set.spawn(async move {
                let res = child.sync_node(op, np_wd).await?;
                // return the Arc<Node> in case that the tmp child is lost
                MyResult::Ok((res, child))
            });
        }

        // join all the child threads
        while let Some(res) = join_set.join_next().await {
            let (res, child) = res.or::<String>(Err("Sync Node : thread join error".into()))??;
            if res.exist() {
                cur_data.children.insert(child.file_name(), child);
            }
        }

        cur_data.pushup_mod().await;
        cur_data.sync_time = remote_data.sync_time;
        cur_data.sync_time.update_one(self.meta.id, op.time);

        if remote_data.status.deleted() && cur_data.mod_time.leq(&remote_data.mod_time) {
            assert!(cur_data.children.is_empty());
            info!("Delete the local dir entirely : {}", self.path.display());
            cur_data.status.set_deleted();
            let wd = cur_data.wd.take().unwrap();
            self.meta
                .watch
                .remove_watch(self.path.as_ref(), &wd)
                .await?;
            self.meta.watch.freeze_watch(&p_wd.clone().unwrap()).await;
            delete_empty_dir(&self.path).await?;
            self.meta.watch.unfreeze_watch(&p_wd.clone().unwrap()).await;
        } else if !cur_data.children.is_empty() && cur_data.status.deleted() {
            info!("Recover the local dir : {}", self.path.display());
            cur_data.status.set_exist();
            assert!(cur_data.wd.is_none());
            cur_data.wd = self.meta.watch.add_watch(&self.path).await;
        }

        return Ok(cur_data.status);
    }

    // sync a single remove file to local
    pub async fn sync_file(
        &self,
        op: SyncOption,
        wd: Option<WatchDescriptor>,
        cur_data: &mut RwLockWriteGuard<'_, NodeData>,
        remote_data: &RemoteData,
    ) -> MyResult<NodeStatus> {
        let wd = wd.expect("WatchDescriptor is required");
        if cur_data.status.exist() && remote_data.status.exist() {
            // both exist
            if cur_data.mod_time.leq(&remote_data.sync_time) {
                // local_m <= remote_s
                info!("Both exist : override the local file");
                self.sync_work(SyncType::Override, op, &wd, cur_data, &remote_data)
                    .await?;
            } else if remote_data.mod_time.leq(&cur_data.sync_time) {
                // local_s >= remote_m
                info!("Both exist : do nothing");
            } else {
                // report conflicts
                info!("Both exist : diverged, conflicts happened");
                conflicts_resolve();
            }
        } else if cur_data.status.exist() || remote_data.status.exist() {
            if remote_data.status.deleted() {
                // remote(deleted) -> local
                if cur_data.create_time.leq_vec(&remote_data.sync_time) {
                    if cur_data.mod_time.leq(&remote_data.sync_time) {
                        info!("Sync from deleted : delete the local file");
                        self.sync_work(SyncType::Delete, op, &wd, cur_data, remote_data)
                            .await?;
                    } else {
                        info!("Sync from deleted : but changes diverged, conflicts happened");
                        conflicts_resolve();
                    }
                } else {
                    info!("Sync from deleted : independent files, do nothing");
                }
            } else {
                // remote -> local(deleted)
                if remote_data.create_time.leq_vec(&cur_data.sync_time) {
                    if remote_data.mod_time.leq(&cur_data.sync_time) {
                        info!("Sync to deleted : do nothing");
                    } else {
                        info!("Sync to deleted : but changes diverged, conflicts happened");
                        conflicts_resolve();
                    }
                } else {
                    info!("Sync to deleted : independent files, create a new copy");
                    self.sync_work(SyncType::Create, op, &wd, cur_data, &remote_data)
                        .await?;
                }
            }
        } else {
            info!("Neither exists : do nothing");
        }
        return Ok(cur_data.status);
    }

    pub async fn sync_work(
        &self,
        ty: SyncType,
        op: SyncOption,
        wd: &WatchDescriptor,
        cur_data: &mut RwLockWriteGuard<'_, NodeData>,
        remote_data: &RemoteData,
    ) -> MyResult<()> {
        assert!(cur_data.wd.is_none());
        self.meta.watch.freeze_watch(wd).await;
        match ty {
            SyncType::Create | SyncType::Override => {
                sync_bytes(&self.path, op.client).await?;
            }
            SyncType::Delete => {
                delete_file(&self.path).await?;
            }
        }
        self.meta.watch.unfreeze_watch(wd).await;

        cur_data.mod_time = remote_data.mod_time.clone();
        cur_data.sync_time = remote_data.sync_time.clone();
        cur_data.sync_time.update_one(self.meta.id, op.time);

        match ty {
            SyncType::Create => {
                cur_data.create_time = remote_data.create_time.clone();
                cur_data.status.set_exist();
            }
            SyncType::Override => {
                cur_data.create_time = remote_data.create_time.clone();
            }
            SyncType::Delete => cur_data.status.set_deleted(),
        }

        Ok(())
    }
}
