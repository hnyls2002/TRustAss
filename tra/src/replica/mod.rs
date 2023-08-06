pub mod checker;
pub mod file_watcher;
pub mod node;
pub mod rep_meta;
pub mod timestamp;
pub mod tree;

use std::{ffi::OsStr, sync::Arc};

use inotify::{Event, EventMask};

use crate::{unwrap_res, MyResult};

use self::{file_watcher::WatchIfc, node::Node, rep_meta::RepMeta};

pub struct Replica {
    pub rep_meta: Arc<RepMeta>,
    pub base_node: Arc<Node>,
    pub watch_ifc: WatchIfc,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModType {
    Create,
    Delete,
    Modify,
    MovedTo,
    MovedFrom,
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

#[derive(Clone)]
pub struct ModOption {
    pub ty: ModType,
    pub time: usize,
    pub name: String,
    pub is_dir: bool,
}

impl Replica {
    pub async fn new(id: i32, watch_ifc: WatchIfc) -> Self {
        let rep_meta = Arc::new(RepMeta::new(id));
        if !rep_meta.check_exist(&rep_meta.prefix) {
            tokio::fs::create_dir(&rep_meta.prefix).await.unwrap();
        } else if !rep_meta.check_is_dir(&rep_meta.prefix) {
            panic!("The root path is not a directory!");
        }
        let base_node = Node::new_base_node(rep_meta.clone(), watch_ifc.clone()).await;
        let base_node = Arc::new(base_node);
        Self {
            rep_meta,
            base_node,
            watch_ifc,
        }
    }

    pub async fn init_all(&self) -> MyResult<()> {
        // init the whole file tree, all inintial is in time 1
        let init_counter = self.rep_meta.add_counter().await;
        let res = self
            .base_node
            .scan_all(init_counter, self.watch_ifc.clone())
            .await;
        unwrap_res!(res);
        self.base_node
            .data
            .write()
            .await
            .mod_time
            .update_singleton(init_counter);
        Ok(())
    }

    pub fn sync_dir(&mut self) -> MyResult<()> {
        todo!()
    }

    pub fn sync_file(&mut self) -> MyResult<()> {
        todo!()
    }

    pub fn clear() {
        todo!();
    }
}

impl Replica {
    pub async fn handle_event(&self, event: &Event<&OsStr>) -> MyResult<()> {
        let path = self
            .watch_ifc
            .query_path(&event.wd)
            .await
            .expect("should have this file watched")
            .clone();
        let walk = self.rep_meta.decompose(&path);
        let name = event
            .name
            .expect("Inotify event name is None")
            .to_string_lossy()
            .to_string();
        let time = self.rep_meta.add_counter().await;
        let op = ModOption {
            ty: ModType::from_mask(&event.mask),
            time,
            name,
            is_dir: event.mask.contains(EventMask::ISDIR),
        };
        let res = self
            .base_node
            .handle_modify(&path, walk, op, self.watch_ifc.clone())
            .await;
        unwrap_res!(res);
        Ok(())
    }
}
