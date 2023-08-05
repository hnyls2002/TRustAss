use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};

use crate::machine::ServeAddr;

use super::{Null, PortCollect, PortNumber};

pub mod controller {
    #![allow(non_snake_case)]
    include!("../protos/controller.rs");
}

pub struct PortCollector {
    pub tx: Sender<(i32, ServeAddr)>,
}

#[tonic::async_trait]
impl PortCollect for PortCollector {
    async fn send_port(&self, req: Request<PortNumber>) -> Result<Response<Null>, Status> {
        let inner = req.into_inner();
        let serve_addr = ServeAddr::new(inner.port as u16);
        self.tx
            .send((inner.id, serve_addr))
            .await
            .expect("failed to send port");
        Ok(Response::new(Null {}))
    }
}
