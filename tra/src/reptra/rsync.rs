use std::path::Path;

use fast_rsync::{diff, Signature, SignatureOptions};
use tonic::{Request, Response, Status};

use crate::{machine::ServeAddr, MyResult};

use super::{peer_server::PeerServer, DiffSource, Patch, ReqRst, Rsync, SyncMsg};

pub const SIG_OPTION: SignatureOptions = SignatureOptions {
    block_size: 4096,
    crypto_hash_size: 64,
};

pub async fn read_bytes(path: impl AsRef<Path>) -> MyResult<Vec<u8>> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err("read bytes failed".into()),
    }
}

#[tonic::async_trait]
impl Rsync for PeerServer {
    async fn fetch_patch(
        &self,
        diff_source: Request<DiffSource>,
    ) -> Result<Response<Patch>, Status> {
        let diff_source = diff_source.into_inner();
        let path = diff_source.path;
        let sig = Signature::deserialize(diff_source.sig).or(Err(Status::invalid_argument(
            "signature deserialized failed",
        )))?;
        let index_sig = sig.index();
        let data = read_bytes(&path)
            .await
            .or(Err(Status::invalid_argument("read bytes failed")))?;
        let mut delta: Vec<u8> = Vec::new();
        diff(&index_sig, &data, &mut delta).or(Err(Status::invalid_argument("diff failed")))?;
        Ok(Response::new(Patch { delta }))
    }

    async fn request_sync(&self, sync_msg: Request<SyncMsg>) -> Result<Response<ReqRst>, Status> {
        let sync_msg = sync_msg.into_inner();
        let path = sync_msg.path;
        let target_addr = ServeAddr::new(sync_msg.port as u16);
        self.rsync_fetch(&path, &target_addr)
            .await
            .or(Err(Status::invalid_argument("rsync fetch failed")))?;
        Ok(Response::new(ReqRst { success: true }))
    }
}
