pub mod greeter;
pub mod port_collect;

pub mod controller {
    include!("../protos/controller.rs");
}

use std::thread;

use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::config::CHANNEL_BUFFER_SIZE;
use crate::config::TRA_PORT;
use crate::info;
use crate::MyResult;

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

type RepThread = thread::JoinHandle<Result<(), std::io::Error>>;

pub struct RepInfo {
    pub thread_handle: RepThread,
    pub port: u16,
}

pub async fn start_centra(rep_num: usize) -> MyResult<()> {
    let greeter = MyGreeter::default();

    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    let port_collector = PortCollector { tx: tx.clone() };

    let server = Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(PortCollectServer::new(port_collector))
        // .serve_with_shutdown("[::]:8080".parse().unwrap(), ctrl_c_singal());
        .serve(format!("[::]:{}", TRA_PORT).parse().unwrap());

    // boot the tra server here
    let handle = tokio::spawn(async {
        let res = server.await;
        println!("");
        info!("Shutting down the tra server...");
        res
    });

    // ----------------- do tra things below -----------------

    // collect the reptra' ports here
    let port_list = collect_ports(rx, rep_num).await;
    for port in port_list {
        info!("Port {} is collected.", port);
    }

    let join_res = handle.await;

    if join_res.is_err() || join_res.unwrap().is_err() {
        return Err("The centra server is down unexpectedly.".into());
    }

    Ok(())
}
