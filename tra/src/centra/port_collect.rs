use tonic::{Request, Response, Status};

use super::{Null, PortCollect, PortNumber};

pub mod controller {
    include!("../protos/controller.rs");
}

pub struct PortCollector {
    pub mac_num: usize,
    pub port_list: Vec<u16>,
}

#[tonic::async_trait]
impl PortCollect for PortCollector {
    async fn send_port(&self, _: Request<PortNumber>) -> Result<Response<Null>, Status> {
        todo!()
    }
}

pub async fn collect_ports(mac_num: usize) -> Vec<u16> {
    todo!()
}
