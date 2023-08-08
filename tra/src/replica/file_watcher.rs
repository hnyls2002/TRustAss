use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use inotify::{Event, Inotify, WatchDescriptor, WatchMask, Watches};
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::{info, MyResult};

lazy_static! {
    static ref WATCH_EVENTS: WatchMask = WatchMask::CREATE
        | WatchMask::DELETE
        | WatchMask::MODIFY
        | WatchMask::MOVED_FROM
        | WatchMask::MOVED_TO;
}

pub struct FileWatcher {
    pub inotify: Inotify,
    pub wd_map: Arc<RwLock<HashMap<WatchDescriptor, PathBuf>>>,
    pub path_map: Arc<RwLock<HashMap<PathBuf, WatchDescriptor>>>,
    pub cnt: i32,
}

#[derive(Clone)]
pub struct WatchIfc {
    watches: Watches,
    wd_map: Arc<RwLock<HashMap<WatchDescriptor, PathBuf>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            inotify: Inotify::init().expect("Failed to initialize inotify"),
            wd_map: Arc::new(RwLock::new(HashMap::new())),
            path_map: Arc::new(RwLock::new(HashMap::new())),
            cnt: 0,
        }
    }

    pub fn get_ifc(&self) -> WatchIfc {
        WatchIfc {
            watches: self.inotify.watches(),
            wd_map: self.wd_map.clone(),
        }
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
    pub async fn add_watch(&self, path: &PathBuf) -> Option<WatchDescriptor> {
        // watching directory is enough
        let mut tmp_watches = self.watches.clone();
        if path.is_dir() {
            info!("add_watches: {}", path.display());
            let wd = tmp_watches.add(path.as_path(), *WATCH_EVENTS).unwrap();
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

    pub async fn query_path(&self, wd: &WatchDescriptor) -> Option<PathBuf> {
        self.wd_map.read().await.get(wd).cloned()
    }
}
