pub mod modification;
pub mod synchronization;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use inotify::WatchDescriptor;
use tokio::sync::RwLock;

use crate::{replica::RepMeta, unwrap_res, MyResult};

use super::{
    file_watcher::WatchIfc,
    timestamp::{SingletonTime, VectorTime},
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
    pub wd: Option<WatchDescriptor>,
}

pub struct Node {
    pub rep_meta: Arc<RepMeta>,
    pub path: Box<PathBuf>,
    pub is_dir: bool,
    pub data: RwLock<NodeData>,
}

// basic methods
impl Node {
    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    // the replica's bedrock
    pub async fn new_base_node(rep_meta: Arc<RepMeta>, mut watch_ifc: WatchIfc) -> Self {
        let path = rep_meta.prefix.clone();
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::new_empty(rep_meta.id),
            sync_time: VectorTime::new_empty(rep_meta.id),
            create_time: SingletonTime::default(),
            status: NodeStatus::Exist,
            wd: watch_ifc.add_watch(&path).await,
        };
        Self {
            path: Box::new(path),
            rep_meta,
            is_dir: true,
            data: RwLock::new(data),
        }
    }
}

impl Node {
    // scan all the files (which are not detected before) in the directory
    #[async_recursion]
    pub async fn scan_all(&self, init_time: usize, watch_ifc: WatchIfc) -> MyResult<()> {
        let static_path = self.path.as_path();
        let mut sub_files = unwrap_res!(tokio::fs::read_dir(static_path)
            .await
            .or(Err("Scan All Error : read dir error")));
        while let Some(sub_file) = sub_files.next_entry().await.unwrap() {
            let child = Arc::new(
                Node::new_from_create(
                    &sub_file.path(),
                    init_time,
                    self.rep_meta.clone(),
                    watch_ifc.clone(),
                )
                .await,
            );
            if child.is_dir {
                child.scan_all(init_time, watch_ifc.clone()).await?;
            }
            self.data
                .write()
                .await
                .children
                .insert(child.file_name(), child);
        }
        Ok(())
    }
}
