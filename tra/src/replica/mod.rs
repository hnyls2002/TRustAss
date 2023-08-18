pub mod file_watcher;
pub mod meta;
pub mod node;
pub mod path_local;
pub mod query;

use std::{ffi::OsStr, sync::Arc};

use inotify::{Event, EventMask};
use tokio::sync::RwLock;

use crate::{
    config::{sync_folder_prefix, RpcChannel},
    reptra::{QueryRes, RsyncClient},
    MyResult,
};

use self::{
    file_watcher::WatchIfc,
    meta::Meta,
    node::{ModOption, ModType, Node, SyncOption},
    path_local::PathLocal,
};

pub struct Replica {
    pub meta: Arc<Meta>,
    pub counter: RwLock<i32>,
    pub base_node: Arc<Node>,
}

impl Replica {
    pub async fn read_counter(&self) -> i32 {
        self.counter.read().await.clone()
    }

    pub async fn add_counter(&self) -> i32 {
        let mut now = self.counter.write().await;
        *now += 1;
        *now
    }

    pub async fn new(id: i32, watch: WatchIfc) -> Self {
        let meta = Arc::new(Meta::new(id, watch));
        let path = PathLocal::new_from_rel(sync_folder_prefix(id), "");
        if !path.exists() {
            tokio::fs::create_dir(&path).await.unwrap();
        } else if !path.is_dir() {
            panic!("The root path is not a directory!");
        }
        let base_node = Node::new_base_node(&meta, path).await;
        let base_node = Arc::new(base_node);
        Self {
            meta,
            counter: RwLock::new(0),
            base_node,
        }
    }

    pub async fn init_all(&self) -> MyResult<()> {
        // init the whole file tree, all inintial is in time 1
        let init_counter = self.add_counter().await;
        self.base_node.scan_all(init_counter).await?;
        self.base_node
            .data
            .write()
            .await
            .mod_time
            .update_one(self.meta.id, init_counter);
        Ok(())
    }

    pub async fn tree(&self, show_detail: bool) {
        self.base_node.sub_tree(show_detail, Vec::new()).await;
    }
}

impl Replica {
    pub async fn handle_event(&self, event: &Event<&OsStr>) -> MyResult<()> {
        let path = self
            .meta
            .watch
            .query_path(&event.wd)
            .await
            .expect("should have this file watched")
            .clone();
        let walk = path.get_walk();
        let op = ModOption {
            ty: ModType::from_mask(&event.mask),
            time: self.add_counter().await,
            name: event.name.unwrap().to_str().unwrap().to_string(),
            is_dir: event.mask.contains(EventMask::ISDIR),
        };
        self.base_node.handle_modify(walk, op).await
    }

    pub async fn handle_query(&self, path: &String) -> MyResult<QueryRes> {
        let path = PathLocal::new_from_rel(&self.base_node.path.prefix(), path);
        let walk = path.get_walk();
        let mut ret = self
            .base_node
            .handle_query(walk)
            .await
            .or(Err("Handle Query : cannot find data"))?;
        ret.sync_time
            .insert(self.meta.id, self.read_counter().await);
        Ok(ret)
    }

    pub async fn handle_sync(
        &self,
        path: &String,
        client: RsyncClient<RpcChannel>,
    ) -> MyResult<()> {
        let path = PathLocal::new_from_rel(self.base_node.path.prefix(), path);
        let walk = path.get_walk();
        let op = SyncOption {
            time: self.add_counter().await,
            client,
        };
        self.base_node.handle_sync(op, walk, None).await?;
        Ok(())
    }

    pub fn clean(&mut self) {
        todo!()
    }
}
