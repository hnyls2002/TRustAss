use fast_rsync::{apply, diff, Signature, SignatureOptions};

use tonic::{Request, Response, Status};

use super::{booter::PeerServer, DiffSource, Patch, ReqRst, Rsync, SyncMsg};

pub fn get_data(path: &String) -> Vec<u8> {
    todo!()
}

#[tonic::async_trait]
impl Rsync for PeerServer {
    async fn fetch_patch(
        &self,
        diff_source: Request<DiffSource>,
    ) -> Result<Response<Patch>, Status> {
        let diff_source = diff_source.into_inner();
        let path = diff_source.path;
        let sig = Signature::deserialize(diff_source.sig).expect("signature deserialized failed");
        let index_sig = sig.index();
        let data = get_data(&path);
        let mut delta: Vec<u8> = Vec::new();
        diff(&index_sig, &data, &mut delta).expect("diff signature failed");

        Ok(Response::new(Patch { delta }))
    }

    async fn request_sync(&self, sync_msg: Request<SyncMsg>) -> Result<Response<ReqRst>, Status> {
        todo!()
    }
}

pub fn demo() {
    let data1 = "hello fuck".as_bytes();
    let data2 = "fuck you".as_bytes();
    let option = SignatureOptions {
        block_size: 1024,
        crypto_hash_size: 8,
    };
    let sig = Signature::calculate(data1, option);
    let index_sig = sig.index();
    let mut buf: Vec<u8> = Vec::new();
    let mut res: Vec<u8> = Vec::new();
    diff(&index_sig, data2, &mut buf).unwrap();
    apply(data1, &buf, &mut res).unwrap();
    println!(
        "rsync demo : {}",
        std::str::from_utf8(res.as_slice()).unwrap()
    );
}
