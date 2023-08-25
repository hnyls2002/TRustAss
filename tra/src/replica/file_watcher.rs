use std::{collections::HashMap, ffi::OsStr, os::raw::c_int, path::Path, sync::Arc};

use inotify::{Event, Inotify, WatchDescriptor, WatchMask, Watches};
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::{
    banner::{BannerOut, LocalBanner},
    debug, MyResult,
};

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
    pub freeze_count_map: Arc<RwLock<HashMap<c_int, usize>>>,
}

#[derive(Clone)]
pub struct WatchIfc {
    watches: Watches,
    wd_map: Arc<RwLock<HashMap<WatchDescriptor, PathLocal>>>,
    freeze_count_map: Arc<RwLock<HashMap<c_int, usize>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            inotify: Inotify::init().expect("Failed to initialize inotify"),
            wd_map: Arc::new(RwLock::new(HashMap::new())),
            freeze_count_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_ifc(&self) -> WatchIfc {
        WatchIfc {
            watches: self.inotify.watches(),
            wd_map: self.wd_map.clone(),
            freeze_count_map: self.freeze_count_map.clone(),
        }
    }

    pub async fn is_freezed(&self, wd: &WatchDescriptor) -> bool {
        self.freeze_count_map
            .read()
            .await
            .get(&wd.get_watch_descriptor_id())
            .map_or(false, |v| *v > 0)
    }

    pub async fn display_event(&self, event: &Event<&OsStr>) {
        let path = self.wd_map.read().await.get(&event.wd).unwrap().clone();
        debug!("Id  : {}", event.wd.get_watch_descriptor_id());
        debug!("Path : {}", path.display());
        debug!("Mask : {:?}", event.mask);
        debug!("Name : {:?}", event.name);
        debug!("==============================================");
    }
}

impl WatchIfc {
    pub async fn add_watch(&self, path: &PathLocal) -> Option<WatchDescriptor> {
        // watching directory is enough
        let mut tmp_watches = self.watches.clone();
        // maybe the file is temporary and deleted immediately
        if !path.exists() {
            BannerOut::warn(format!("Path does not exist : {}", path.display()));
        }
        if path.is_dir() {
            LocalBanner::new_watch(path);
            let wd = tmp_watches.add(path, *WATCH_EVENTS).unwrap();
            self.wd_map.write().await.insert(wd.clone(), path.clone());
            Some(wd)
        } else {
            None
        }
    }

    pub async fn remove_watch(&self, path: impl AsRef<Path>, wd: &WatchDescriptor) -> MyResult<()> {
        LocalBanner::remove_watch(path);
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
        self.freeze_count_map
            .write()
            .await
            .entry(wd.get_watch_descriptor_id())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub async fn unfreeze_watch(&self, wd: &WatchDescriptor) {
        let mut mp = self.freeze_count_map.write().await;
        mp.entry(wd.get_watch_descriptor_id())
            .and_modify(|v| *v -= 1);
        if mp.get(&wd.get_watch_descriptor_id()).unwrap() == &0 {
            mp.remove(&wd.get_watch_descriptor_id());
        }
    }
}
