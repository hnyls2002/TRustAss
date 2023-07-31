use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use inotify::{Event, Inotify, WatchDescriptor, WatchMask};
use lazy_static::lazy_static;

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
    pub wd_map: HashMap<WatchDescriptor, PathBuf>,
    pub cnt: usize,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            inotify: Inotify::init().expect("Failed to initialize inotify"),
            wd_map: HashMap::new(),
            cnt: 0,
        }
    }

    pub fn add_watch(&mut self, path: &PathBuf) -> Option<WatchDescriptor> {
        // watching directory is enough
        if path.is_dir() {
            info!("add_watches: {}", path.display());
            let wd = self
                .inotify
                .watches()
                .add(path.as_path(), *WATCH_EVENTS)
                .unwrap();
            self.wd_map.insert(wd.clone(), path.clone());
            Some(wd)
        } else {
            None
        }
    }

    pub fn remove_watch(&mut self, path: &PathBuf, wd: &WatchDescriptor) -> MyResult<()> {
        info!("remove_watches: {}", path.display());
        self.wd_map.remove(wd);
        self.inotify
            .watches()
            .remove((*wd).clone())
            .or(Err("Failed to remove watch"))?;
        Ok(())
    }

    pub fn display_event(&mut self, event: &Event<&OsStr>) {
        self.cnt = self.cnt + 1;
        let path = self.wd_map.get(&event.wd).unwrap();
        println!("Event: {}", self.cnt);
        println!("Path : {}", path.display());
        println!("Mask : {:?}", event.mask);
        println!("Name : {:?}", event.name);
        println!("==============================================");
    }
}
