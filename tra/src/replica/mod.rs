pub mod file_watcher;
pub mod meta;
pub mod node;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use inotify::{Event, EventMask};
use tokio::sync::RwLock;

use crate::{
    config::RpcChannel,
    reptra::{QueryRes, RsyncClient},
    unwrap_res, MyResult,
};

use self::{
    file_watcher::WatchIfc,
    meta::Meta,
    node::{ModOption, ModType, Node, SyncOption},
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
        if !meta.check_exist(&meta.prefix) {
            tokio::fs::create_dir(&meta.prefix).await.unwrap();
        } else if !meta.check_is_dir(&meta.prefix) {
            panic!("The root path is not a directory!");
        }
        let base_node = Node::new_base_node(&meta).await;
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
        let res = self.base_node.scan_all(init_counter).await;
        unwrap_res!(res);
        self.base_node
            .data
            .write()
            .await
            .mod_time
            .update_singleton(init_counter);
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
        let walk = self.meta.decompose_absolute(&path);
        let op = ModOption {
            ty: ModType::from_mask(&event.mask),
            time: self.add_counter().await,
            name: event.name.unwrap().to_str().unwrap().to_string(),
            is_dir: event.mask.contains(EventMask::ISDIR),
        };
        self.base_node.handle_modify(walk, op).await
    }

    pub async fn handle_query(&self, path: impl AsRef<Path>) -> MyResult<QueryRes> {
        let path = PathBuf::from(path.as_ref());
        let walk = self.meta.decompose_absolute(&path);
        self.base_node.handle_query(walk).await
    }

    pub async fn handle_sync(
        &self,
        path: impl AsRef<Path>,
        is_dir: bool,
        client: RsyncClient<RpcChannel>,
    ) -> MyResult<()> {
        let op = SyncOption {
            time: self.add_counter().await,
            is_dir,
            client,
        };
        let walk = self.meta.decompose_absolute(&PathBuf::from(path.as_ref()));
        self.base_node.handle_sync(op, walk).await?;
        Ok(())
    }

    pub fn clean(&mut self) {
        todo!()
    }
}
