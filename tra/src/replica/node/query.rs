use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;

use crate::{
    replica::{node::NodeStatus, rep_meta::RepMeta},
    reptra::QueryRes,
    MyResult,
};

use super::{Node, NodeData};

impl QueryRes {
    pub fn new_exist(rep_meta: &Arc<RepMeta>, data: &NodeData) -> Self {
        Self {
            id: rep_meta.id,
            deleted: false,
            create_time: data.create_time.extract_time(),
            mod_time: data.mod_time.extract_hashmap(),
            sync_time: data.sync_time.extract_hashmap(),
        }
    }

    pub fn new_deleted(rep_meta: &Arc<RepMeta>, data: &NodeData) -> Self {
        Self {
            id: rep_meta.id,
            deleted: true,
            create_time: 0,
            mod_time: HashMap::new(),
            sync_time: data.sync_time.extract_hashmap(),
        }
    }
}

impl Node {
    #[async_recursion]
    pub async fn handle_query(&self, mut walk: Vec<String>) -> MyResult<QueryRes> {
        let cur_data = self.data.read().await;

        // deleted : return directly
        if cur_data.status == NodeStatus::Deleted {
            return Ok(QueryRes::new_deleted(&self.rep_meta, &cur_data));
        }

        if !walk.is_empty() {
            // not the target node yet
            let child_name = walk.pop().unwrap();
            if let Some(child) = cur_data.children.get(&child_name) {
                child.handle_query(walk).await
            } else {
                // if the child node does not exist, return father's sync time
                Ok(QueryRes::new_deleted(&self.rep_meta, &cur_data))
            }
        } else {
            // the target node, and it exists
            Ok(QueryRes::new_exist(&self.rep_meta, &cur_data))
        }
    }
}
