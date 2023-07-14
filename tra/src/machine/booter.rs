use rand::Rng;
use std::net::TcpListener as StdListener;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tonic::{
    transport::{server::TcpIncoming, Server},
    Request,
};

use crate::centra::{PortCollectClient, PortNumber};

use super::{rsync::Synchronizer, RsyncServer, SyncMsg};

type HandleType = JoinHandle<Result<(), tonic::transport::Error>>;

#[derive(Default, Clone, Copy)]
pub struct ReplicaServer {
    pub port: u16,
}

pub async fn boot_server(
    tonic_channel: tonic::transport::Channel,
    request_tx: &Sender<SyncMsg>,
) -> HandleType {
    let mut rng = rand::thread_rng();
    let (port, handle) = loop {
        let port = rng.gen_range(49152..=65535);
        let listener = StdListener::bind(format!("[::]:{}", port));
        let server_inner = Synchronizer {
            request_tx: request_tx.clone(),
        };
        if let Ok(listener) = listener {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
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
        .send_port(Request::new(PortNumber { port }))
        .await
        .expect("failed to send port number");

    handle
}
