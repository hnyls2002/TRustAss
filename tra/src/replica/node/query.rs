use std::{path::PathBuf, sync::Arc};

use async_recursion::async_recursion;

use crate::{
    replica::{node::NodeStatus, rep_meta::RepMeta},
    reptra::QueryRes,
    MyResult,
};

use super::{Node, NodeData};

impl QueryRes {
    pub fn new_deleted(rep_meta: Arc<RepMeta>, data: &NodeData) -> Self {
        todo!()
    }
}

impl Node {
    #[async_recursion]
    pub async fn handle_query(&self, path: &PathBuf, mut walk: Vec<String>) -> MyResult<QueryRes> {
        // if the target node is deleted, return directly
        if self.data.read().await.status == NodeStatus::Deleted {}
        // not the target node yet
        if !walk.is_empty() {
            let cur_data = self.data.read().await;
            let child_name = walk.pop().unwrap();
            if let Some(child) = cur_data.children.get(&child_name) {
            } else {
            }
        }
        todo!()
    }
}
