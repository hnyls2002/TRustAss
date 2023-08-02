pub mod greeter;
pub mod port_collect;

pub mod controller {
    #![allow(non_snake_case)]
    include!("../protos/controller.rs");
}

use crate::{
    config::{MpscReceiver, MpscSender, ServiceHandle},
    machine::ServeAddr,
};
use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::config::CHANNEL_BUFFER_SIZE;
use crate::info;
use crate::replica::Replica;

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
    pub replica: Replica,
    pub addr_tx: MpscSender<ServeAddr>,
    pub addr_rx: MpscReceiver<ServeAddr>,
    pub reptra_addrs: Vec<ServeAddr>,
    pub service_handle: Option<ServiceHandle>,
}

impl Centra {
    pub fn new(serve_addr: &ServeAddr) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        Self {
            serve_addr: serve_addr.clone(),
            replica: Replica::new(0),
            addr_tx: tx,
            addr_rx: rx,
            reptra_addrs: Vec::new(),
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
            info!("Shutting down the tra server...");
            res
        }));
    }

    pub async fn collect_ports(&mut self, rep_num: usize) {
        for _ in 0..rep_num {
            if let Some(serve_addr) = self.addr_rx.recv().await {
                self.reptra_addrs.push(serve_addr);
                info!("Port {} is collected.", serve_addr.port());
            } else {
                panic!("The port collect channel is closed unexpectedly.");
            }
        }
    }
}
