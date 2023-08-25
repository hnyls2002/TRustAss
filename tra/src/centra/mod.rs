pub mod greeter;
pub mod port_collect;

pub mod controller {
    #![allow(non_snake_case)]
    include!("../protos/controller.rs");
}

use std::collections::HashMap;

use crate::{
    banner::BannerOut,
    config::{MpscReceiver, MpscSender, ServiceHandle},
    machine::ServeAddr,
};
use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::config::CHANNEL_BUFFER_SIZE;

pub use controller::{
    greeter_client::GreeterClient,
    greeter_server::{Greeter, GreeterServer},
    port_collect_client::PortCollectClient,
    port_collect_server::{PortCollect, PortCollectServer},
    HelloReply, HelloRequest, Null, PortNumber,
};

pub use greeter::MyGreeter;
pub use port_collect::PortCollector;

pub struct Centra {
    pub serve_addr: ServeAddr,
    pub addr_tx: MpscSender<(i32, ServeAddr)>,
    pub addr_rx: MpscReceiver<(i32, ServeAddr)>,
    pub id_map: HashMap<i32, ServeAddr>,
    pub service_handle: Option<ServiceHandle>,
}

impl Centra {
    pub fn new(serve_addr: &ServeAddr) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        Self {
            serve_addr: serve_addr.clone(),
            addr_tx: tx,
            addr_rx: rx,
            id_map: HashMap::new(),
            service_handle: None,
        }
    }

    pub async fn start_services(&mut self) {
        let greeter = MyGreeter::default();
        let port_collector = PortCollector {
            tx: self.addr_tx.clone(),
        };

        let server = Server::builder()
            .add_service(GreeterServer::new(greeter))
            .add_service(PortCollectServer::new(port_collector))
            // .serve_with_shutdown("[::]:8080".parse().unwrap(), ctrl_c_singal());
            .serve(self.serve_addr.addr().parse().unwrap());

        // boot the tra server here
        self.service_handle = Some(tokio::spawn(async {
            let res = server.await;
            println!("");
            println!("Shutting down the tra server...");
            res
        }));
    }

    pub async fn collect_ports(&mut self, rep_num: usize) {
        for _ in 0..rep_num {
            if let Some((id, serve_addr)) = self.addr_rx.recv().await {
                self.id_map.insert(id, serve_addr);
                BannerOut::check(format!(
                    "id : {}, port : {} collected",
                    id,
                    serve_addr.port()
                ));
            } else {
                panic!("The port collect channel is closed unexpectedly.");
            }
        }
    }

    pub fn get_addr(&self, id: i32) -> ServeAddr {
        self.id_map[&id].clone()
    }
}
