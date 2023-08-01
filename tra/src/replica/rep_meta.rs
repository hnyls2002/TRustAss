use std::path::PathBuf;

use tokio::sync::RwLock;

use crate::config::TMP_PATH;

use super::node::NodeStatus;

pub struct RepMeta {
    pub(super) id: u16,
    pub(super) prefix: PathBuf,
    pub(super) counter: RwLock<usize>,
}

impl RepMeta {
    pub fn new(port: u16) -> Self {
        Self {
            id: port,
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