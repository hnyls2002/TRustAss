use rand::Rng;
use std::net::TcpListener as StdListener;
use tokio::task::JoinHandle;
use tonic::{
    transport::{server::TcpIncoming, Server},
    Request,
};

use crate::{
    centra::{PortCollectClient, PortNumber},
    replica::Replica,
};

use super::RsyncServer;

type HandleType = JoinHandle<Result<(), tonic::transport::Error>>;

pub struct PeerServer {
    pub rep: Replica,
}

pub async fn boot_server(tonic_channel: tonic::transport::Channel) -> HandleType {
    let mut rng = rand::thread_rng();
    let (port, handle) = loop {
        let port = rng.gen_range(49152..=65535);
        let listener = StdListener::bind(format!("[::]:{}", port));
        if let Ok(listener) = listener {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
            let server_inner = PeerServer {
                rep: Replica::new(port),
            };
            let server = Server::builder()
                .add_service(RsyncServer::new(server_inner))
                // .serve_with_incoming_shutdown(incoming, ctrl_c_singal());
                .serve_with_incoming(incoming);
            let handle = tokio::spawn(async { server.await });
            break (port, handle);
        }
    };

    // send the port to centra server here
    let mut port_sender = PortCollectClient::new(tonic_channel);

    port_sender
        .send_port(Request::new(PortNumber { port: port as i32 }))
        .await
        .expect("failed to send port number");

    handle
}
