use rand::Rng;
use std::net::TcpListener as StdListener;
use tokio::task::JoinHandle;
use tonic::transport::{server::TcpIncoming, Server};

use crate::{centra::MyGreeter, hello::greeter_server::GreeterServer};

type HandleType = JoinHandle<Result<(), tonic::transport::Error>>;

struct ReplicaServer {}

pub fn boot_server() -> (HandleType, u16) {
    let mut rng = rand::thread_rng();
    let server = MyGreeter::default();
    loop {
        let port = rng.gen_range(49152..=65535);
        let listener = StdListener::bind(format!("[::]:{}", port));
        if let Ok(listener) = listener {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
            let server = Server::builder()
                .add_service(GreeterServer::new(server.clone()))
                // .serve_with_incoming_shutdown(incoming, ctrl_c_singal());
                .serve_with_incoming(incoming);
            let handle = tokio::spawn(async { server.await });
            return (handle, port);
        }
    }
}
