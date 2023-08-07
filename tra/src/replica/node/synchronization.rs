use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use tokio::sync::RwLock;
use tonic::Request;

use crate::{
    config::RpcChannel,
    replica::{
        file_watcher::WatchIfc,
        rep_meta::RepMeta,
        timestamp::{SingletonTime, VectorTime},
    },
    reptra::{QueryReq, QueryRes, RsyncClient},
    MyResult,
};

use super::{Node, NodeData, NodeStatus};

#[derive(Clone)]
pub struct SyncOption {
    pub time: i32,
    pub is_dir: bool,
    pub client: RsyncClient<RpcChannel>,
}

impl SyncOption {
    pub async fn query_path(&mut self, path: &PathBuf) -> MyResult<QueryRes> {
        Ok(self
            .client
            .query(Request::new(QueryReq {
                path: path.to_str().unwrap().to_string(),
            }))
            .await
            .or(Err("query failed"))?
            .into_inner())
    }

    // pub async fn copy_file(&self, path : &PathBuf ) -> MyResult<()> {
    //     let data = self.replica.rep_meta.read_bytes(path).await?;
    //     let sig = Signature::calculate(&data, SIG_OPTION);
    //     let request = FetchPatchReq {
    //         path: path.clone(),
    //         sig: Vec::from(sig.serialized()),
    //     };
    //     let channel = self.get_channel(target_addr).await?;
    //     let mut client = RsyncClient::new(channel);
    //     let patch = client
    //         .fetch_patch(request)
    //         .await
    //         .or(Err("fetch patch failed"))?;
    //     let delta = patch.into_inner().delta;
    //     let mut out: Vec<u8> = Vec::new();
    //     apply(&data, &delta, &mut out).or(Err("apply failed"))?;
    //     self.replica.rep_meta.sync_bytes(path, out).await?;
    //     info!("The size of data is {}", data.len());
    //     info!("The size of patch is {}", delta.len());
    //     Ok(())
    //     todo!()
    // }
}

impl Node {
    // the new temporary node which is not exist in the file system
    // when any ground sycnchronization happens, we will make it exist
    pub async fn new_tmp(
        op: &SyncOption,
        rep_meta: Arc<RepMeta>,
        tmp_path: &PathBuf,
        sync_time: VectorTime,
    ) -> Self {
        let data = NodeData {
            children: HashMap::new(),
            mod_time: VectorTime::new_empty(rep_meta.id),
            sync_time,
            create_time: SingletonTime::new(rep_meta.id, 0),
            status: NodeStatus::Deleted,
            wd: None,
        };
        Self {
            rep_meta: rep_meta.clone(),
            path: Box::new(tmp_path.clone()),
            is_dir: op.is_dir,
            data: RwLock::new(data),
        }
    }
}

// recursive methods
impl Node {
    #[async_recursion]
    pub async fn handle_sync(
        &self,
        mut op: SyncOption,
        mut walk: Vec<String>,
        watch_ifc: WatchIfc,
    ) -> MyResult<()> {
        if !walk.is_empty() {
            // not the target node yet
            let mut cur_data = self.data.write().await;
            let child_name = walk.pop().unwrap();
            if let Some(child) = cur_data.children.get(&child_name) {
                // can find the child
                child.handle_sync(op, walk, watch_ifc.clone()).await?;
            } else {
                // child is deleted or not exist
                let tmp_node = Node::new_tmp(
                    &op,
                    self.rep_meta.clone(),
                    &self.path.join(child_name),
                    cur_data.sync_time.clone(),
                )
                .await;
                tmp_node.handle_sync(op, walk, watch_ifc).await?;
            }
        } else {
            // find the target file(dir) to be synchronized
            if op.is_dir {
                self.sync_dir(op.clone()).await?;
            } else {
                self.sync_file(op.clone()).await?;
            }
        }
        Ok(())
    }
}

// the sync implementation
impl Node {
    pub async fn sync_dir(&self, mut op: SyncOption) -> MyResult<()> {
        todo!()
    }

    pub async fn sync_file(&self, mut op: SyncOption) -> MyResult<()> {
        let data = self.data.write().await;
        let query_res = op.query_path(&self.path).await?;

        if data.status == NodeStatus::Exist && query_res.deleted == false {
            // both exist
            if data.mod_time.leq(&query_res.sync_time) {
                // do nothing
                return Ok(());
            } else if data.sync_time.geq(&query_res.mod_time) {
                // copy the file
                return Ok(());
            } else {
                // report conflicts
                todo!()
            }
        } else if data.status == NodeStatus::Exist || query_res.deleted == false {
            // one exists, the other is deleted
        }
        todo!()
    }

    /*
    single file
    A B(deleted)
    if c_A <= s_B {
        it is B that deletes the file
        if m_A <= s_B{
            A -> B : do nothing
            B -> A : delete A
        }
        else {
            report conflicts
        }
    }
    else {
        they are independent
        A -> B : copy A to B
        B -> A : do nothing
    }
     */

    /*
    directories
    A B(deleted)
    when a dir was created, its sync time is at least its creation time
    if c_A <= s_B {
        it is B that deletes the file
        if m_A <= s_B{
            A -> B : skip the dir
            B -> A : traverse each subnode, to delete them
        }
        else {
            traverse each subnode, handle next
        }
    }
    else {
        they are independent
        A -> B : traverse each subnode, to copy them
        B -> A : skip the dir
    }
    */
}
