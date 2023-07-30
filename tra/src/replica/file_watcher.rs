use std::{collections::HashMap, ffi::OsStr, sync::Arc};

use async_recursion::async_recursion;
use inotify::{Event, EventMask, Inotify, WatchMask};
use lazy_static::lazy_static;

use crate::{error, info};

use super::node::Node;

lazy_static! {
    static ref WATCH_EVENTS: WatchMask = WatchMask::CREATE
        | WatchMask::DELETE
        | WatchMask::MODIFY
        | WatchMask::MOVED_FROM
        | WatchMask::MOVED_TO;
}

pub struct FileWatcher {
    pub inotify: Inotify,
    pub fd_map: HashMap<i32, Arc<Node>>,
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

    pub fn add_file(&mut self, node: &Arc<Node>) {
        // watching directory is enough
        // self.inotify.watches().remove(wd)
        if node.is_dir {
            info!("add_watches: {:?}", node.path);
            let fd = self
                .inotify
                .watches()
                .add(node.path.as_path(), *WATCH_EVENTS)
                .unwrap()
                .get_watch_descriptor_id();
            self.fd_map.insert(fd, node.clone());
        }
    }

    #[async_recursion]
    pub async fn bind_watches_recursive(&mut self, node: &Arc<Node>) {
        self.add_file(node);
        let data = node.data.read().await;
        for (_, child) in data.children.iter() {
            self.bind_watches_recursive(child).await;
        }
    }
}

impl FileWatcher {
    pub fn display(&mut self, event: &Event<&OsStr>) {
        self.cnt = self.cnt + 1;
        let fd = event.wd.get_watch_descriptor_id();
        let node = self.fd_map.get(&fd).unwrap();
        println!("Event: {}", self.cnt);
        println!("Path : {}", node.path.display());
        println!("Mask : {:?}", event.mask);
        println!("Name : {:?}", event.name);
        println!("===============================");
    }

    pub async fn handle_event(&mut self, event: &Event<&OsStr>) {
        let fd = event.wd.get_watch_descriptor_id();
        let node = self.fd_map.get(&fd).unwrap().clone();
        let time = node.rep_meta.add_counter().await;
        let mask = event.mask;
        let name = event
            .name
            .expect("Inotify event name is None")
            .to_string_lossy()
            .to_string();
        if mask.contains(EventMask::CREATE) {
            node.handle_create(&name, time, Arc::downgrade(&node)).await;

            // add watch for the new file
            let child = node.data.read().await.children.get(&name).unwrap().clone();
            self.add_file(&child);
        } else if mask.contains(EventMask::DELETE) {
            node.handle_delete(&name, time).await;
        } else if mask.contains(EventMask::MODIFY) {
            node.handle_modify(time).await;
        } else if mask.contains(EventMask::MOVED_TO) {
            // when file or dir move to here, first build the node
            node.handle_moved_to(&name, time, Arc::downgrade(&node))
                .await;
            // then add watch for the whole dir
            let child = node.data.read().await.children.get(&name).unwrap().clone();
            self.bind_watches_recursive(&child).await;
        } else {
            error!("Unknown event mask: {:?}", mask);
        }
    }
}
