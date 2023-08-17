use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::Path,
    sync::Arc,
};

use inotify::{Event, Inotify, WatchDescriptor, WatchMask, Watches};
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::{info, MyResult};

use super::path_local::PathLocal;

lazy_static! {
    static ref WATCH_EVENTS: WatchMask = WatchMask::CREATE
        | WatchMask::DELETE
        | WatchMask::MODIFY
        | WatchMask::MOVED_FROM
        | WatchMask::MOVED_TO;
}

pub struct FileWatcher {
    pub inotify: Inotify,
    pub wd_map: Arc<RwLock<HashMap<WatchDescriptor, PathLocal>>>,
    pub freeze_set: Arc<RwLock<HashSet<WatchDescriptor>>>,
    pub cnt: i32,
}

#[derive(Clone)]
pub struct WatchIfc {
    watches: Watches,
    wd_map: Arc<RwLock<HashMap<WatchDescriptor, PathLocal>>>,
    freeze_set: Arc<RwLock<HashSet<WatchDescriptor>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            inotify: Inotify::init().expect("Failed to initialize inotify"),
            wd_map: Arc::new(RwLock::new(HashMap::new())),
            freeze_set: Arc::new(RwLock::new(HashSet::new())),
            cnt: 0,
        }
    }

    pub fn get_ifc(&self) -> WatchIfc {
        WatchIfc {
            watches: self.inotify.watches(),
            wd_map: self.wd_map.clone(),
            freeze_set: self.freeze_set.clone(),
        }
    }

    pub async fn is_freezed(&self, wd: &WatchDescriptor) -> bool {
        self.freeze_set.read().await.contains(wd)
    }

    pub async fn display_event(&mut self, event: &Event<&OsStr>) {
        self.cnt = self.cnt + 1;
        let path = self.wd_map.read().await.get(&event.wd).unwrap().clone();
        println!("Event: {}", self.cnt);
        println!("Path : {}", path.display());
        println!("Mask : {:?}", event.mask);
        println!("Name : {:?}", event.name);
        println!("==============================================");
    }
}

impl WatchIfc {
    pub async fn add_watch(&self, path: &PathLocal) -> Option<WatchDescriptor> {
        // watching directory is enough
        let mut tmp_watches = self.watches.clone();
        assert!(path.exists(), "Path not exist");
        if path.is_dir() {
            info!("add_watches: {}", path.display());
            let wd = tmp_watches.add(path, *WATCH_EVENTS).unwrap();
            self.wd_map.write().await.insert(wd.clone(), path.clone());
            Some(wd)
        } else {
            None
        }
    }

    pub async fn remove_watch(&self, path: impl AsRef<Path>, wd: &WatchDescriptor) -> MyResult<()> {
        info!("remove_watches: {}", path.as_ref().display());
        self.wd_map.write().await.remove(wd);
        let mut tmp_watches = self.watches.clone();
        tmp_watches
            .remove((*wd).clone())
            .or(Err("Watch already removed"))?;
        Ok(())
    }

    pub async fn query_path(&self, wd: &WatchDescriptor) -> Option<PathLocal> {
        self.wd_map.read().await.get(wd).cloned()
    }

    pub async fn freeze_watch(&self, wd: &WatchDescriptor) {
        self.freeze_set.write().await.insert(wd.clone());
    }

    pub async fn unfreeze_watch(&self, wd: &WatchDescriptor) {
        self.freeze_set.write().await.remove(wd);
    }
}
