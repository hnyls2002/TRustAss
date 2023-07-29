use std::{collections::HashMap, ffi::OsStr, sync::Arc};

use async_recursion::async_recursion;
use inotify::{Inotify, WatchMask};
use lazy_static::lazy_static;

use crate::{config::CHANNEL_BUFFER_SIZE, debug, warn};

use super::{node::Node, Replica};

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
    pub fn add_file(&mut self, node: &Arc<Node>) {
        let fd = self
            .inotify
            .watches()
            .add(node.path.as_path(), *WATCH_EVENTS)
            .unwrap()
            .get_watch_descriptor_id();
        self.fd_map.insert(fd, node.clone());
    }

    pub async fn new_from_replica(replica: &Replica) -> Self {
        let mut fw = Self {
            inotify: Inotify::init().expect("Failed to initialize inotify"),
            fd_map: HashMap::new(),
            cnt: 0,
        };
        Self::bind_watches(&mut fw, &replica.trees_collect).await;
        fw
    }

    #[async_recursion]
    pub async fn bind_watches(fw: &mut FileWatcher, node: &Arc<Node>) {
        warn!("add_watches: {:?}", node.path);
        fw.add_file(node);
        let data = node.data.read().await;
        for (_, child) in data.children.iter() {
            Self::bind_watches(fw, &child).await;
        }
    }

    pub fn display(&mut self, event: inotify::Event<&OsStr>) {
        self.cnt = self.cnt + 1;
        let fd = event.wd.get_watch_descriptor_id();
        let node = self.fd_map.get(&fd).unwrap();
        println!("Event: {}", self.cnt);
        println!("Path : {}", node.path.display());
        println!("Mask : {:?}", event.mask);
        println!("Name : {:?}", event.name);
        println!("===============================");
    }

    pub fn work(&mut self) -> ! {
        let mut buffer = [0; CHANNEL_BUFFER_SIZE];
        loop {
            let events = self.inotify.read_events_blocking(buffer.as_mut()).unwrap();
            for event in events {
                self.display(event);
            }
        }
    }
}

pub fn file_watch_test(dir_path: &String) {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    let path = std::path::Path::new(dir_path);
    std::fs::create_dir_all(path).unwrap();

    debug!("All events can be watched {:?}", WatchMask::ALL_EVENTS);

    inotify.watches().add(path, WatchMask::ALL_EVENTS).unwrap();

    let mut buffer = [0; 1024];

    loop {
        debug!("Waiting for events");
        let events = inotify.read_events_blocking(buffer.as_mut()).unwrap();

        for event in events {
            println!("{:?}", event);
        }
    }
}
