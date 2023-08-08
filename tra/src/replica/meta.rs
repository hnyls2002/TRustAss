use std::path::{Path, PathBuf};

use fast_rsync::{apply, Signature};
use tokio::io::AsyncWriteExt;

use crate::{
    config::{RpcChannel, SIG_OPTION, TMP_PATH},
    info,
    reptra::{FetchPatchReq, RsyncClient},
    MyResult,
};

use super::node::NodeStatus;

pub struct Meta {
    pub(super) id: i32,
    pub(super) prefix: PathBuf,
}

impl Meta {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            prefix: PathBuf::from(format!("{}replica-{}", TMP_PATH, id)),
        }
    }

    pub fn to_absolute(&self, relative: impl AsRef<Path>) -> PathBuf {
        let mut ret = self.prefix.clone();
        ret.push(relative);
        ret
    }

    pub fn check_exist(&self, relative: &PathBuf) -> bool {
        self.to_absolute(relative).exists()
    }

    pub fn check_is_dir(&self, relative: &PathBuf) -> bool {
        self.to_absolute(relative).is_dir()
    }

    pub fn get_status(&self, relative: &PathBuf) -> NodeStatus {
        self.check_exist(relative)
            .then(|| NodeStatus::Exist)
            .unwrap_or(NodeStatus::Deleted)
    }

    pub fn decompose_absolute(&self, path: &PathBuf) -> Vec<String> {
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

    pub async fn read_bytes(&self, path: impl AsRef<Path>) -> MyResult<Vec<u8>> {
        let file_entry = self.to_absolute(path).canonicalize();
        if let Ok(path_exist) = file_entry {
            match tokio::fs::read(path_exist).await {
                Ok(bytes) => return Ok(bytes),
                Err(_) => Err("Read Bytes : read bytes failed".into()),
            }
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn write_bytes(
        &self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
    ) -> MyResult<()> {
        let full_path = self.to_absolute(path);
        let mut file = match full_path.canonicalize() {
            Ok(path_exist) => tokio::fs::OpenOptions::new()
                .write(true)
                .open(path_exist)
                .await
                .or(Err("Sync Bytes : open file failed"))?,
            Err(_) => tokio::fs::File::create(full_path)
                .await
                .or(Err("Sync Bytes : create file failed"))?,
        };
        file.write_all(data.as_ref())
            .await
            .or(Err("Sync Bytes : write bytes to file failed"))?;
        file.flush()
            .await
            .or(Err("Sync Bytes : flush file failed"))?;
        Ok(())
    }

    pub async fn sync_bytes(
        &self,
        path: impl AsRef<Path>,
        mut client: RsyncClient<RpcChannel>,
    ) -> MyResult<()> {
        let data = self.read_bytes(path.as_ref()).await?;
        let sig = Signature::calculate(&data, SIG_OPTION);
        let request = FetchPatchReq {
            path: path.as_ref().to_str().unwrap().to_string(),
            sig: Vec::from(sig.serialized()),
        };
        let patch = client
            .fetch_patch(request)
            .await
            .or(Err("Sync Bytes : fetch patch failed"))?;
        let delta = patch.into_inner().delta;
        let mut out: Vec<u8> = Vec::new();
        apply(&data, &delta, &mut out).or(Err("Sync Bytes : apply failed"))?;
        self.write_bytes(path, out).await?;
        info!("The size of data is {}", data.len());
        info!("The size of patch is {}", delta.len());
        Ok(())
    }
}
