pub mod greeter;
pub mod port_collect;

pub mod controller {
    #![allow(non_snake_case)]
    include!("../protos/controller.rs");
}

use crate::config::{MpscReceiver, MpscSender, ServiceHandle};
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
    pub port: u16,
    pub replica: Replica,
    pub port_tx: MpscSender<u16>,
    pub port_rx: MpscReceiver<u16>,
    pub port_list: Vec<u16>,
    pub service_handle: Option<ServiceHandle>,
}

impl Centra {
    pub fn new(port: u16) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        Self {
            port,
            replica: Replica::new(port),
            port_tx: tx,
            port_rx: rx,
            port_list: Vec::new(),
            service_handle: None,
        }
    }

    pub async fn start_services(&mut self) {
        let greeter = MyGreeter::default();
        let port_collector = PortCollector {
            tx: self.port_tx.clone(),
        };

        let server = Server::builder()
            .add_service(GreeterServer::new(greeter))
            .add_service(PortCollectServer::new(port_collector))
            // .serve_with_shutdown("[::]:8080".parse().unwrap(), ctrl_c_singal());
            .serve(format!("[::]:{}", self.port).parse().unwrap());

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
            if let Some(port) = self.port_rx.recv().await {
                self.port_list.push(port);
                info!("Port {} is collected.", port);
            } else {
                panic!("The port collect channel is closed unexpectedly.");
            }
        }
    }
}
