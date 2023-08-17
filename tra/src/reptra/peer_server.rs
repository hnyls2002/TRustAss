use std::{collections::HashMap, sync::Arc};

use fast_rsync::{diff, Signature};

use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::{
    config::RpcChannel,
    machine::{channel_connect, ServeAddr},
    replica::{meta::read_bytes, path_local::PathLocal, Replica},
    reptra::FetchPatchReq,
    MyResult,
};

use super::{BoolResult, Patch, QueryReq, QueryRes, Rsync, RsyncClient, SyncReq};

pub struct PeerServer {
    pub replica: Arc<Replica>,
    pub channels: Arc<RwLock<HashMap<ServeAddr, RpcChannel>>>,
}

impl PeerServer {
    pub async fn get_channel(&self, serve_addr: &ServeAddr) -> MyResult<RpcChannel> {
        let mut inner = self.channels.write().await;
        if inner.get(&serve_addr).is_some() {
            return Ok(inner.get(&serve_addr).unwrap().clone());
        }
        let channel = channel_connect(serve_addr).await?;
        inner.insert(serve_addr.clone(), channel.clone());
        return Ok(channel);
    }
}

#[tonic::async_trait]
impl Rsync for PeerServer {
    /// get the signature and diff, send back delta
    async fn fetch_patch(&self, req: Request<FetchPatchReq>) -> Result<Response<Patch>, Status> {
        let inner = req.into_inner();
        let path = PathLocal::new_from_rel(&self.replica.base_node.path.prefix(), &inner.path_rel);
        let sig = Signature::deserialize(inner.sig).or(Err(Status::invalid_argument(
            "signature deserialized failed",
        )))?;
        let index_sig = sig.index();
        let data = read_bytes(&path)
            .await
            .map_err(|e| Status::invalid_argument(e))?;
        let mut delta: Vec<u8> = Vec::new();
        diff(&index_sig, &data, &mut delta).or(Err(Status::invalid_argument("diff failed")))?;
        Ok(Response::new(Patch { delta }))
    }

    /// query the info of one file(dir)
    async fn query(&self, req: Request<QueryReq>) -> Result<Response<QueryRes>, Status> {
        let res = self
            .replica
            .handle_query(&req.into_inner().path_rel)
            .await
            .map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(res))
    }

    async fn request_sync(&self, req: Request<SyncReq>) -> Result<Response<BoolResult>, Status> {
        let inner = req.into_inner();
        let query_channel = self
            .get_channel(&ServeAddr::new(inner.port as u16))
            .await
            .map_err(|e| Status::invalid_argument(e))?;
        let client = RsyncClient::new(query_channel);
        self.replica
            .handle_sync(&inner.path_rel, client)
            .await
            .map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(BoolResult { success: true }))
    }
}
