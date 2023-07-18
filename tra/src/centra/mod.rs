pub mod greeter;
pub mod port_collect;

pub mod controller {
    include!("../protos/controller.rs");
}

use std::thread;

use std::io::Error as IoError;
use std::io::Result as IoResult;

use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::config::CHANNEL_BUFFER_SIZE;
use crate::info;

pub use controller::{
    greeter_client::GreeterClient,
    greeter_server::{Greeter, GreeterServer},
    port_collect_client::PortCollectClient,
    port_collect_server::{PortCollect, PortCollectServer},
    HelloReply, HelloRequest, Null, PortNumber,
};

pub use greeter::MyGreeter;
pub use port_collect::PortCollector;

use self::port_collect::collect_ports;

type RepThread = thread::JoinHandle<Result<(), IoError>>;

pub struct RepInfo {
    pub thread_handle: RepThread,
    pub port: u16,
}

pub async fn start_centra(rep_num: usize) -> IoResult<()> {
    let greeter = MyGreeter::default();

    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    let port_collector = PortCollector { tx: tx.clone() };

    let server = Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(PortCollectServer::new(port_collector))
        // .serve_with_shutdown("[::]:8080".parse().unwrap(), ctrl_c_singal());
        .serve("[::]:8080".parse().unwrap());

    // boot the tra server here
    let handle = tokio::spawn(async {
        server.await.unwrap();
        println!("");
        info!("Shutting down the tra server...");
    });

    // ----------------- do tra things below -----------------

    // collect the reptra' ports here
    let port_list = collect_ports(rx, rep_num).await;
    for port in port_list {
        info!("Port {} is collected.", port);
    }

    handle.await?;

    Ok(())
}
