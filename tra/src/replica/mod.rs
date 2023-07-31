pub mod checker;
pub mod file_sync;
pub mod file_watcher;
pub mod node;
pub mod timestamp;
pub mod tree;

use std::{ffi::OsStr, path::PathBuf, sync::Arc};

use inotify::{Event, EventMask};
use tokio::sync::RwLock;

use crate::{
    config::{CHANNEL_BUFFER_SIZE, TMP_PATH},
    unwrap_res, MyResult,
};

use self::{
    file_watcher::FileWatcher,
    node::{Node, NodeStatus},
};

pub struct RepMeta {
    pub port: u16,
    pub prefix: PathBuf,
    pub counter: RwLock<usize>,
}

impl RepMeta {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            prefix: PathBuf::from(format!("{}{}", TMP_PATH, port)),
            counter: RwLock::new(0),
        }
    }

    pub fn to_absolute(&self, relative: &PathBuf) -> PathBuf {
        let mut ret = self.prefix.clone();
        ret.push(relative);
        ret
    }

    pub fn to_relative(&self, absolute: &PathBuf) -> Option<PathBuf> {
        absolute
            .clone()
            .strip_prefix(&self.prefix)
            .map_or(None, |f| Some(f.to_path_buf()))
    }

    pub fn check_exist(&self, relative: &PathBuf) -> bool {
        let mut path = self.prefix.clone();
        path.push(relative);
        path.exists()
    }

    pub fn check_is_dir(&self, relative: &PathBuf) -> bool {
        let mut path = self.prefix.clone();
        path.push(relative);
        path.is_dir()
    }

    pub fn get_status(&self, relative: &PathBuf) -> NodeStatus {
        self.check_exist(relative)
            .then(|| NodeStatus::Exist)
            .unwrap_or(NodeStatus::Deleted)
    }

    pub fn decompose(&self, path: &PathBuf) -> Vec<String> {
        let mut tmp_path = path.clone();
        let mut ret: Vec<String> = Vec::new();
        while tmp_path.file_name().is_some() {
            if tmp_path == self.prefix {
                break;
            }
            ret.push(tmp_path.file_name().unwrap().to_str().unwrap().to_string());
            tmp_path.pop();
        }
        ret
    }

    pub async fn read_counter(&self) -> usize {
        self.counter.read().await.clone()
    }

    pub async fn add_counter(&self) -> usize {
        let mut now = self.counter.write().await;
        *now += 1;
        *now
    }
}

pub struct Replica {
    pub rep_meta: Arc<RepMeta>,
    pub trees_collect: Arc<Node>,
    pub file_watcher: FileWatcher,
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
    pub fn new(port: u16) -> Self {
        let rep_meta = Arc::new(RepMeta::new(port));
        let mut file_watcher = FileWatcher::new();
        let trees_collect = Arc::new(Node::new_trees_collect(rep_meta.clone(), &mut file_watcher));
        Self {
            rep_meta,
            file_watcher,
            trees_collect,
        }
    }

    pub async fn init_file_trees(&mut self) -> MyResult<()> {
        // init the whole file tree, all inintial is in time 1
        let init_counter = self.rep_meta.add_counter().await;
        let trees_collect_weak = Arc::downgrade(&self.trees_collect);
        let res = self
            .trees_collect
            .scan_all(init_counter, trees_collect_weak, &mut self.file_watcher)
            .await;
        unwrap_res!(res);
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

// handling the watching stuff
impl Replica {
    pub async fn watching(&mut self) -> ! {
        let mut buffer = [0; CHANNEL_BUFFER_SIZE];
        loop {
            let events = self
                .file_watcher
                .inotify
                .read_events_blocking(buffer.as_mut())
                .unwrap();
            for event in events {
                if event.mask != EventMask::IGNORED {
                    self.file_watcher.display_event(&event);
                    self.handle_event(&event).await.unwrap();
                }
            }
            self.tree().await;
        }
    }

    pub async fn handle_event(&mut self, event: &Event<&OsStr>) -> MyResult<()> {
        let path = self
            .file_watcher
            .wd_map
            .get(&event.wd)
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
        let cur_weak = Arc::downgrade(&self.trees_collect);
        let res = self
            .trees_collect
            .handle_event(&path, walk, op, cur_weak, &mut self.file_watcher)
            .await;
        unwrap_res!(res);
        Ok(())
    }
}
