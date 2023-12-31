use std::sync::Arc;

use fast_rsync::{apply, Signature};
use tokio::{io::AsyncWriteExt, sync::Mutex};

use crate::{
    config::{RpcChannel, SIG_OPTION},
    reptra::{FetchPatchReq, RsyncClient},
    MyResult,
};

use super::{file_watcher::WatchIfc, path_local::PathLocal};

pub struct Meta {
    pub(super) id: i32,
    pub(super) watch: WatchIfc,
    pub(super) c_lock: Arc<Mutex<()>>,
}

impl Meta {
    pub fn new(id: i32, watch: WatchIfc, c_lock: Arc<Mutex<()>>) -> Self {
        Self { id, watch, c_lock }
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
            .or(Err("Write Bytes : open file failed"))?
    } else {
        let mut parent = path.clone();
        parent.pop().ok_or("Write Bytes : get parent path failed")?;
        create_dir_all(&parent).await?;
        tokio::fs::File::create(path)
            .await
            .or(Err("Write Bytes : create file failed"))?
    };
    file.write_all(data.as_ref())
        .await
        .or(Err("Write Bytes : write bytes to file failed"))?;
    file.flush()
        .await
        .or(Err("Write Bytes : flush file failed"))?;
    Ok(())
}

pub async fn get_sync_bytes(
    path: &PathLocal,
    mut client: RsyncClient<RpcChannel>,
) -> MyResult<Vec<u8>> {
    let data = read_bytes(path).await?;
    let sig = Signature::calculate(&data, SIG_OPTION);
    let request = FetchPatchReq {
        path_rel: path.to_rel(),
        sig: Vec::from(sig.serialized()),
    };
    let patch = client
        .fetch_patch(request)
        .await
        .map_err(|e| "unable to fetch patch".to_string() + &e.to_string())?;
    let delta = patch.into_inner().delta;
    let mut synced: Vec<u8> = Vec::new();
    apply(&data, &delta, &mut synced).or(Err("Sync Bytes : apply failed"))?;
    // info!("The size of data is {}, the size of patch is {}", data.len(), delta.len());
    Ok(synced)
}

pub async fn sync_bytes(path: &PathLocal, client: RsyncClient<RpcChannel>) -> MyResult<()> {
    let synced = get_sync_bytes(path, client).await?;
    write_bytes(&path, synced).await?;
    Ok(())
}

pub async fn delete_file(path: &PathLocal) -> MyResult<()> {
    tokio::fs::remove_file(path)
        .await
        .or(Err("Delete File : remove file failed"))?;
    Ok(())
}

pub async fn delete_empty_dir(path: &PathLocal) -> MyResult<()> {
    // remove_dir will fail if the directory is not empty
    tokio::fs::remove_dir(path)
        .await
        .or(Err("Delete Empty Dir : remove dir failed".into()))
}

pub async fn create_dir_all(path: &PathLocal) -> MyResult<()> {
    tokio::fs::create_dir_all(path)
        .await
        .or(Err("Write Bytes : create dir failed".into()))
}
