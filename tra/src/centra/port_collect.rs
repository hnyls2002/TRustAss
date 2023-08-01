use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};

use super::{Null, PortCollect, PortNumber};

pub mod controller {
    #![allow(non_snake_case)]
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
