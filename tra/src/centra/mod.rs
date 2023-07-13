pub mod greeter;
pub mod port_collect;

pub mod controller {
    include!("../protos/controller.rs");
}

use std::thread;

use std::io::Error as IoError;
use std::io::Result as IoResult;

use tonic::transport::Server;

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

type MacThread = thread::JoinHandle<Result<(), IoError>>;

pub struct MacInfo {
    pub thread_handle: MacThread,
    pub port: u16,
}

pub async fn start_tra() -> IoResult<()> {
    let server = MyGreeter::default();

    let server = Server::builder()
        .add_service(GreeterServer::new(server))
        // .serve_with_shutdown("[::]:8080".parse().unwrap(), ctrl_c_singal());
        .serve("[::]:8080".parse().unwrap());

    server.await.unwrap();

    println!("");
    info!("Shutting down the tra server...");

    Ok(())
}
