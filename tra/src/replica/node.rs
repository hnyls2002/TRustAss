use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use inotify::{EventMask, WatchDescriptor};
use tokio::sync::RwLock;
use tonic::Request;

use crate::{
    config::RpcChannel,
    replica::Meta,
    reptra::{QueryReq, QueryRes, RsyncClient},
    timestamp::{SingletonTime, VectorTime},
    unwrap_res, MyResult,
};

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
    pub path: Box<PathBuf>,
    pub is_dir: bool,
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
    pub fn new_exist(meta: &Arc<Meta>, data: &NodeData) -> Self {
        Self {
            id: meta.id,
            deleted: false,
            create_time: data.create_time.extract_time(),
            mod_time: data.mod_time.extract_hashmap(),
            sync_time: data.sync_time.extract_hashmap(),
        }
    }

    pub fn new_deleted(meta: &Arc<Meta>, data: &NodeData) -> Self {
        Self {
            id: meta.id,
            deleted: true,
            create_time: 0,
            mod_time: HashMap::new(),
            sync_time: data.sync_time.extract_hashmap(),
        }
    }
}

impl SyncOption {
    pub async fn query_path(&mut self, path: &PathBuf) -> MyResult<QueryRes> {
        Ok(self
            .client
            .query(Request::new(QueryReq {
                path: path.to_str().unwrap().to_string(),
            }))
            .await
            .or(Err("query failed"))?
            .into_inner())
    }

    // pub async fn copy_file(&self, path : &PathBuf ) -> MyResult<()> {
    //     let data = self.replica.rep_meta.read_bytes(path).await?;
    //     let sig = Signature::calculate(&data, SIG_OPTION);
    //     let request = FetchPatchReq {
    //         path: path.clone(),
    //         sig: Vec::from(sig.serialized()),
    //     };
    //     let channel = self.get_channel(target_addr).await?;
    //     let mut client = RsyncClient::new(channel);
    //     let patch = client
    //         .fetch_patch(request)
    //         .await
    //         .or(Err("fetch patch failed"))?;
    //     let delta = patch.into_inner().delta;
    //     let mut out: Vec<u8> = Vec::new();
    //     apply(&data, &delta, &mut out).or(Err("apply failed"))?;
    //     self.replica.rep_meta.sync_bytes(path, out).await?;
    //     info!("The size of data is {}", data.len());
    //     info!("The size of patch is {}", delta.len());
    //     Ok(())
    //     todo!()
    // }
}

impl Node {
    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
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
        if self.is_dir {
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

        undeleted.sort_by(|a, b| (a.is_dir, a.file_name()).cmp(&(b.is_dir, b.file_name())));

        for child in &undeleted {
            let now_flag = child.file_name() == undeleted.last().unwrap().file_name();
            let mut new_is_last = is_last.clone();
            new_is_last.push(now_flag);
            child.sub_tree(show_detail, new_is_last).await;
        }
    }

    // the replica's bedrock
    pub async fn new_base_node(meta: &Arc<Meta>) -> Self {
        let path = meta.prefix.clone();
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::new_empty(meta.id),
            sync_time: VectorTime::new_empty(meta.id),
            create_time: SingletonTime::default(),
            status: NodeStatus::Exist,
            wd: meta.watch.add_watch(&path).await,
        };
        Self {
            path: Box::new(path),
            meta: meta.clone(),
            is_dir: true,
            data: RwLock::new(data),
        }
    }

    pub async fn new_from_create(path: &PathBuf, time: i32, meta: &Arc<Meta>) -> Self {
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
            is_dir: meta.check_is_dir(path),
            meta: meta.clone(),
            data: RwLock::new(data),
        }
    }

    // the new temporary node which is not exist in the file system
    // when any ground sycnchronization happens, we will make it exist
    pub async fn new_tmp(
        op: &SyncOption,
        meta: &Arc<Meta>,
        tmp_path: &PathBuf,
        sync_time: &VectorTime,
    ) -> Self {
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::new_empty(meta.id),
            sync_time: sync_time.clone(),
            create_time: SingletonTime::new(meta.id, 0),
            status: NodeStatus::Deleted,
            wd: None,
        };
        Self {
            meta: meta.clone(),
            path: Box::new(tmp_path.clone()),
            is_dir: op.is_dir,
            data: RwLock::new(data),
        }
    }
}

// recursive operation on node and node's data
impl Node {
    // scan all the files (which are not detected before) in the directory
    #[async_recursion]
    pub async fn scan_all(&self, init_time: i32) -> MyResult<()> {
        let static_path = self.path.as_path();
        let mut sub_files = unwrap_res!(tokio::fs::read_dir(static_path)
            .await
            .or(Err("Scan All Error : read dir error")));
        while let Some(sub_file) = sub_files.next_entry().await.unwrap() {
            let child =
                Arc::new(Node::new_from_create(&sub_file.path(), init_time, &self.meta).await);
            if child.is_dir {
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
            cur_data.mod_time.update_singleton(op.time);
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
                    data.mod_time.update_singleton(op.time);
                }
                ModType::Modify => {
                    let mut data = self.data.write().await;
                    let modified = data
                        .children
                        .get(&op.name)
                        .ok_or("Modify Error : Node not found when handling Modify event")?;
                    modified.modify_self(op.time).await?;
                    data.mod_time.update_singleton(op.time);
                }
                ModType::MovedFrom => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found when handling MovedTo event")?;
                    deleted.delete_moved_from(op.time).await?;
                    data.mod_time.update_singleton(op.time);
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
            return Ok(QueryRes::new_deleted(&self.meta, &cur_data));
        }

        if !walk.is_empty() {
            // not the target node yet
            let child_name = walk.pop().unwrap();
            if let Some(child) = cur_data.children.get(&child_name) {
                child.handle_query(walk).await
            } else {
                // if the child node does not exist, return father's sync time
                Ok(QueryRes::new_deleted(&self.meta, &cur_data))
            }
        } else {
            // the target node, and it exists
            Ok(QueryRes::new_exist(&self.meta, &cur_data))
        }
    }

    // always sync remote -> local
    #[async_recursion]
    pub async fn handle_sync(&self, mut op: SyncOption, mut walk: Vec<String>) -> MyResult<()> {
        if !walk.is_empty() {
            // not the target node yet
            let mut cur_data = self.data.write().await;
            let child_name = walk.pop().unwrap();
            if let Some(child) = cur_data.children.get(&child_name) {
                // can find the child
                child.handle_sync(op, walk).await?;
            } else {
                // child is deleted or not exist
                let tmp_node = Node::new_tmp(
                    &op,
                    &self.meta,
                    &self.path.join(child_name),
                    &cur_data.sync_time,
                )
                .await;
                tmp_node.handle_sync(op, walk).await?;
            }
        } else {
            // find the target file(dir) to be synchronized
            if op.is_dir {
                self.sync_dir(op.clone()).await?;
            } else {
                self.sync_file(op.clone()).await?;
            }
        }
        Ok(())
    }
}

impl Node {
    pub async fn create_child(&self, name: &String, time: i32) -> MyResult<()> {
        let child_path = self.path.join(name);
        let child = Arc::new(Node::new_from_create(&child_path, time, &self.meta).await);
        if child.is_dir {
            let res = child.scan_all(time).await;
            unwrap_res!(res);
        }
        let mut parent_data = self.data.write().await;
        parent_data.children.insert(name.clone(), child);
        parent_data.mod_time.update_singleton(time);
        Ok(())
    }

    pub async fn modify_self(&self, time: i32) -> MyResult<()> {
        let mut data = self.data.write().await;
        data.mod_time.update_singleton(time);
        data.sync_time.update_singleton(time);
        Ok(())
    }

    // just one file, actually removed in file system
    // and the watch descriptor is automatically removed
    pub async fn delete_rm(&self, time: i32) -> MyResult<()> {
        let mut data = self.data.write().await;
        // the file system cannot delete a directory that is not empty
        for (_, child) in data.children.iter() {
            if child.data.read().await.status == NodeStatus::Exist {
                return Err("Delete Error : Directory not empty".into());
            }
        }
        // clear the mod time && update the sync time
        data.mod_time.clear();
        data.sync_time.update_singleton(time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            if let Ok(_) = self.meta.watch.remove_watch(self.path.as_ref(), &wd).await {
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
    pub async fn delete_moved_from(&self, time: i32) -> MyResult<()> {
        let mut data = self.data.write().await;
        if data.status == NodeStatus::Deleted {
            return Ok(());
        }
        for (_, child) in data.children.iter() {
            unwrap_res!(child.delete_moved_from(time).await);
        }
        data.mod_time.clear();
        data.sync_time.update_singleton(time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            self.meta
                .watch
                .remove_watch(self.path.as_ref(), &wd)
                .await?;
        } else if self.is_dir {
            return Err("Delete Error : No Watch Descriptor".into());
        }
        Ok(())
    }

    // sync a remote folder -> local folder
    #[async_recursion]
    pub async fn sync_dir(&self, mut op: SyncOption) -> MyResult<()> {
        let data = self.data.write().await;
        let remote = op.query_path(&self.path).await?;
        if (remote.deleted && data.status == NodeStatus::Deleted)
            || (!remote.deleted && data.sync_time.geq(&remote.mod_time))
        {
            // skip
            return Ok(());
        } else {
            for (_, child) in data.children.iter() {
                child.sync_dir(op.clone()).await?;
            }
        }
        todo!()
    }

    // sync a single remove file to local
    pub async fn sync_file(&self, mut op: SyncOption) -> MyResult<()> {
        let data = self.data.write().await;
        let remote = op.query_path(&self.path).await?;

        if data.status == NodeStatus::Exist && !remote.deleted {
            if data.mod_time.leq(&remote.sync_time) {
                // local_m <= remote_s
                // override the local file
                return Ok(());
            } else if data.sync_time.geq(&remote.mod_time) {
                // local_s >= remote_m
                return Ok(());
            } else {
                // report conflicts
                todo!()
            }
        } else if data.status == NodeStatus::Exist || remote.deleted == false {
            if remote.deleted {
                // remote(deleted) -> local
                if data.create_time.leq_vec(&remote.sync_time) {
                    if data.mod_time.leq(&remote.sync_time) {
                        // delete the local file
                        todo!();
                    } else {
                        // report conflicts
                        todo!()
                    }
                } else {
                    // do nothing
                    return Ok(());
                }
            } else {
                // remote -> local(deleted)
                let (id, time) = (remote.id, remote.create_time);
                if data.sync_time.geq_singleton(id, time) {
                    if data.sync_time.geq(&remote.mod_time) {
                        // do nothing
                        return Ok(());
                    } else {
                        // report conflicts
                        todo!()
                    }
                } else {
                    // copy the file
                    self.meta.sync_bytes(self.path.as_path(), op.client).await?;
                    return Ok(());
                }
            }
        }
        todo!()
    }

    pub async fn override_sync(&self) -> MyResult<()> {
        todo!()
    }

    pub async fn create_sync(&self) -> MyResult<()> {
        todo!()
    }

    pub async fn delete_sync(&self) -> MyResult<()> {
        todo!()
    }

    /*
    single file
    A B(deleted)
    if c_A <= s_B {
        it is B that deletes the file
        if m_A <= s_B{
            A -> B : do nothing
            B -> A : delete A
        }
        else {
            report conflicts
        }
    }
    else {
        they are independent
        A -> B : copy A to B
        B -> A : do nothing
    }
     */

    /*
    directories
    A B(deleted)
    when a dir was created, its sync time is at least its creation time
    if c_A <= s_B {
        it is B that deletes the file
        if m_A <= s_B{
            A -> B : skip the dir
            B -> A : traverse each subnode, to delete them
        }
        else {
            traverse each subnode, handle next
        }
    }
    else {
        they are independent
        A -> B : traverse each subnode, to copy them
        B -> A : skip the dir
    }
    */
}
