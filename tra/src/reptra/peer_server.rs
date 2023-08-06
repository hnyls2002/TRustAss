use std::{collections::HashMap, sync::Arc};

use fast_rsync::{apply, Signature};

use tokio::sync::RwLock;

use crate::{
    config::RpcChannel,
    info,
    machine::{channel_connect, ServeAddr},
    replica::Replica,
    reptra::FetchPatchReq,
    MyResult,
};

use super::{rsync::SIG_OPTION, RsyncClient};

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

    pub async fn rsync_fetch(&self, path: &String, target_addr: &ServeAddr) -> MyResult<()> {
        let data = self.replica.rep_meta.read_bytes(path).await?;
        let sig = Signature::calculate(&data, SIG_OPTION);
        let request = FetchPatchReq {
            path: path.clone(),
            sig: Vec::from(sig.serialized()),
        };
        let channel = self.get_channel(target_addr).await?;
        let mut client = RsyncClient::new(channel);
        let patch = client
            .fetch_patch(request)
            .await
            .or(Err("fetch patch failed"))?;
        let delta = patch.into_inner().delta;
        let mut out: Vec<u8> = Vec::new();
        apply(&data, &delta, &mut out).or(Err("apply failed"))?;
        self.replica.rep_meta.sync_bytes(path, out).await?;
        info!("The size of data is {}", data.len());
        info!("The size of patch is {}", delta.len());
        Ok(())
    }
}
