use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};

use crate::machine::ServeAddr;

use super::{Null, PortCollect, PortNumber};

pub mod controller {
    #![allow(non_snake_case)]
    include!("../protos/controller.rs");
}

pub struct PortCollector {
    pub tx: Sender<ServeAddr>,
}

#[tonic::async_trait]
impl PortCollect for PortCollector {
    async fn send_port(&self, req: Request<PortNumber>) -> Result<Response<Null>, Status> {
        let serve_addr = ServeAddr::new(req.into_inner().port as u16);
        self.tx.send(serve_addr).await.expect("failed to send port");
        Ok(Response::new(Null {}))
    }
}
