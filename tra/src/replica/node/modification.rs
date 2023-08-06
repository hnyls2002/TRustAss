use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use tokio::sync::RwLock;

use crate::{
    replica::{
        file_watcher::WatchIfc,
        rep_meta::RepMeta,
        timestamp::{SingletonTime, VectorTime},
        ModOption, ModType,
    },
    unwrap_res, MyResult,
};

use super::{Node, NodeData, NodeStatus};

// basic methods
impl Node {
    pub async fn new_from_create(
        path: &PathBuf,
        time: usize,
        rep_meta: Arc<RepMeta>,
        mut watch_ifc: WatchIfc,
    ) -> Self {
        let create_time = SingletonTime::new(rep_meta.id, time);
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::from_singleton_time(&create_time),
            sync_time: VectorTime::from_singleton_time(&create_time),
            create_time,
            status: NodeStatus::Exist,
            wd: watch_ifc.add_watch(path).await,
        };
        Node {
            path: Box::new(path.clone()),
            is_dir: rep_meta.check_is_dir(path),
            rep_meta,
            data: RwLock::new(data),
        }
    }
}

// direct operation on node and node's data
impl Node {
    pub async fn create(&self, name: &String, time: usize, watch_ifc: WatchIfc) -> MyResult<()> {
        let child_path = self.path.join(name);
        let child = Arc::new(
            Node::new_from_create(&child_path, time, self.rep_meta.clone(), watch_ifc.clone())
                .await,
        );
        if child.is_dir {
            let res = child.scan_all(time, watch_ifc).await;
            unwrap_res!(res);
        }
        let mut parent_data = self.data.write().await;
        parent_data.children.insert(name.clone(), child);
        parent_data.mod_time.update_singleton(time);
        Ok(())
    }

    pub async fn modify(&self, time: usize) -> MyResult<()> {
        let mut data = self.data.write().await;
        data.mod_time.update_singleton(time);
        data.sync_time.update_singleton(time);
        Ok(())
    }

    // just one file, actually removed in file system
    // and the watch descriptor is automatically removed
    pub async fn delete_rm(&self, time: usize, mut watch_ifc: WatchIfc) -> MyResult<()> {
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
            if let Ok(_) = watch_ifc.remove_watch(&self.path, &wd).await {
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
    pub async fn delete_moved_from(&self, time: usize, mut watch_ifc: WatchIfc) -> MyResult<()> {
        let mut data = self.data.write().await;
        if data.status == NodeStatus::Deleted {
            return Ok(());
        }
        for (_, child) in data.children.iter() {
            unwrap_res!(child.delete_moved_from(time, watch_ifc.clone()).await);
        }
        data.mod_time.clear();
        data.sync_time.update_singleton(time);
        data.status = NodeStatus::Deleted;
        if data.wd.is_some() {
            let wd = data.wd.take().unwrap();
            watch_ifc.remove_watch(&self.path, &wd).await?;
        } else if self.is_dir {
            return Err("Delete Error : No Watch Descriptor".into());
        }
        Ok(())
    }
}

// recursive operation on node and node's data
impl Node {
    #[async_recursion]
    pub async fn handle_modify(
        &self,
        path: &PathBuf,
        mut walk: Vec<String>,
        op: ModOption,
        watch_ifc: WatchIfc,
    ) -> MyResult<()> {
        // not the target node yet
        if !walk.is_empty() {
            let mut cur_data = self.data.write().await;
            let child_name = walk.pop().unwrap();
            let child = cur_data
                .children
                .get(&child_name)
                .ok_or("Event Handling Error : Node not found along the path")?;
            child
                .handle_modify(path, walk, op.clone(), watch_ifc)
                .await?;
            cur_data.mod_time.update_singleton(op.time);
        } else {
            match op.ty {
                ModType::Create | ModType::MovedTo => {
                    // create method : from parent node handling it
                    self.create(&op.name, op.time, watch_ifc).await?;
                }
                ModType::Delete => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found when handling Delete Event")?;
                    deleted.delete_rm(op.time, watch_ifc).await?;
                    data.mod_time.update_singleton(op.time);
                }
                ModType::Modify => {
                    let mut data = self.data.write().await;
                    let modified = data
                        .children
                        .get(&op.name)
                        .ok_or("Modify Error : Node not found when handling Modify event")?;
                    modified.modify(op.time).await?;
                    data.mod_time.update_singleton(op.time);
                }
                ModType::MovedFrom => {
                    let mut data = self.data.write().await;
                    let deleted = data
                        .children
                        .get(&op.name)
                        .ok_or("Delete Error : Node not found when handling MovedTo event")?;
                    deleted.delete_moved_from(op.time, watch_ifc).await?;
                    data.mod_time.update_singleton(op.time);
                }
            };
        };
        Ok(())
    }
}
