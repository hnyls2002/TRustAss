use tokio::sync::mpsc::{Receiver, Sender};
use tonic::{Request, Response, Status};

use super::{Null, PortCollect, PortNumber};

pub mod controller {
    include!("../protos/controller.rs");
}

pub struct PortCollector {
    pub tx: Sender<u16>,
}

#[tonic::async_trait]
impl PortCollect for PortCollector {
    async fn send_port(&self, req: Request<PortNumber>) -> Result<Response<Null>, Status> {
        let port = req.into_inner().port;
        self.tx
            .send(port as u16)
            .await
            .expect("failed to send port");
        Ok(Response::new(Null {}))
    }
}

pub async fn collect_ports(mut rx: Receiver<u16>, rep_num: usize) -> Vec<u16> {
    let mut ret = Vec::new();
    for _ in 0..rep_num {
        if let Some(port) = rx.recv().await {
            ret.push(port);
        }
    }
    ret
}
