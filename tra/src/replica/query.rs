use crate::{
    reptra::QueryRes,
    timestamp::{SingletonTime, VectorTime},
};

use super::node::{NodeData, NodeStatus};

pub struct RemoteData {
    pub children: Vec<String>,
    pub mod_time: VectorTime,
    pub sync_time: VectorTime,
    pub create_time: SingletonTime,
    pub status: NodeStatus,
}

impl QueryRes {
    pub fn from_data(data: &NodeData, is_dir: bool) -> Self {
        Self {
            deleted: data.status.eq(&NodeStatus::Deleted),
            create_id: data.create_time.create_id(),
            create_time: data.create_time.time(),
            mod_time: data.mod_time.clone().into(),
            sync_time: data.sync_time.clone().into(),
            children: data.children.iter().map(|(k, _)| k.clone()).collect(),
            is_dir,
        }
    }

    pub fn to_data(&self) -> (RemoteData, bool) {
        let data = RemoteData {
            children: self.children.clone(),
            mod_time: self.mod_time.clone().into(),
            sync_time: self.sync_time.clone().into(),
            create_time: SingletonTime::new(self.create_id, self.create_time),
            status: if self.deleted {
                NodeStatus::Deleted
            } else {
                NodeStatus::Exist
            },
        };
        (data, self.is_dir)
    }
}
