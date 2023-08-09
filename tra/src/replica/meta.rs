use fast_rsync::{apply, Signature};
use inotify::WatchDescriptor;
use tokio::io::AsyncWriteExt;

use crate::{
    config::{RpcChannel, SIG_OPTION},
    info,
    reptra::{FetchPatchReq, RsyncClient},
    MyResult,
};

use super::{file_watcher::WatchIfc, path_local::PathLocal};

pub struct Meta {
    pub(super) id: i32,
    pub(super) watch: WatchIfc,
}

impl Meta {
    pub fn new(id: i32, watch: WatchIfc) -> Self {
        Self { id, watch }
    }

    pub async fn sync_bytes(
        &self,
        path: &PathLocal,
        wd: &WatchDescriptor,
        mut client: RsyncClient<RpcChannel>,
    ) -> MyResult<()> {
        let data = read_bytes(path).await?;
        let sig = Signature::calculate(&data, SIG_OPTION);
        let request = FetchPatchReq {
            path_rel: path.to_rel(),
            sig: Vec::from(sig.serialized()),
        };
        let patch = client
            .fetch_patch(request)
            .await
            .or(Err("Sync Bytes : fetch patch failed"))?;
        let delta = patch.into_inner().delta;
        let mut out: Vec<u8> = Vec::new();
        apply(&data, &delta, &mut out).or(Err("Sync Bytes : apply failed"))?;
        // get the new data done, first remove the watcher on current file
        self.watch.freeze_watch(wd).await;
        write_bytes(&path, out).await?;
        self.watch.unfreeze_watch(wd).await;
        info!("The size of data is {}", data.len());
        info!("The size of patch is {}", delta.len());
        Ok(())
    }
}

pub async fn read_bytes(path: &PathLocal) -> MyResult<Vec<u8>> {
    if path.exists() {
        match tokio::fs::read(path).await {
            Ok(bytes) => return Ok(bytes),
            Err(_) => Err("Read Bytes : read bytes failed".into()),
        }
    } else {
        Ok(Vec::new())
    }
}

pub async fn write_bytes(path: &PathLocal, data: impl AsRef<[u8]>) -> MyResult<()> {
    let mut file = if path.exists() {
        tokio::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .await
            .or(Err("Sync Bytes : open file failed"))?
    } else {
        tokio::fs::File::create(path)
            .await
            .or(Err("Sync Bytes : create file failed"))?
    };
    file.write_all(data.as_ref())
        .await
        .or(Err("Sync Bytes : write bytes to file failed"))?;
    file.flush()
        .await
        .or(Err("Sync Bytes : flush file failed"))?;
    Ok(())
}
