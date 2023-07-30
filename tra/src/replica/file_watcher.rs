use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use inotify::{Event, Inotify, WatchMask};
use lazy_static::lazy_static;

use crate::info;

lazy_static! {
    static ref WATCH_EVENTS: WatchMask = WatchMask::CREATE
        | WatchMask::DELETE
        | WatchMask::MODIFY
        | WatchMask::MOVED_FROM
        | WatchMask::MOVED_TO;
}

pub struct FileWatcher {
    pub inotify: Inotify,
    pub fd_map: HashMap<i32, PathBuf>,
    pub cnt: usize,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            inotify: Inotify::init().expect("Failed to initialize inotify"),
            fd_map: HashMap::new(),
            cnt: 0,
        }
    }

    pub fn add_watch(&mut self, path: &PathBuf) {
        // watching directory is enough
        if path.is_dir() {
            info!("add_watches: {}", path.display());
            let fd = self
                .inotify
                .watches()
                .add(path.as_path(), *WATCH_EVENTS)
                .unwrap()
                .get_watch_descriptor_id();
            self.fd_map.insert(fd, path.clone());
        }
    }

    // #[async_recursion]
    // pub async fn bind_watches_recursive(&mut self, node: &Arc<Node>) {
    //     self.watch_path(&node.path);
    //     let data = node.data.read().await;
    //     for (_, child) in data.children.iter() {
    //         self.bind_watches_recursive(child).await;
    //     }
    // }

    pub fn display_event(&mut self, event: &Event<&OsStr>) {
        self.cnt = self.cnt + 1;
        let fd = event.wd.get_watch_descriptor_id();
        let path = self.fd_map.get(&fd).unwrap();
        println!("Event: {}", self.cnt);
        println!("Path : {}", path.display());
        println!("Mask : {:?}", event.mask);
        println!("Name : {:?}", event.name);
        println!("==============================================");
    }
}
