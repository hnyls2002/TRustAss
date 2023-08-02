pub mod peer_server;

pub mod peer {
    #![allow(non_snake_case)]
    include!("../protos/peer.rs");
}

use crate::{
    centra::{GreeterClient, HelloRequest, PortCollectClient, PortNumber},
    config::ServiceHandle,
    info,
    machine::{channel_connect, get_listener, ServeAddr},
    replica::Replica,
    MyResult,
};

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tonic::{transport::Server, Request};

use self::peer_server::PeerServer;
pub use peer::{
    rsync_client::RsyncClient,
    rsync_server::{Rsync, RsyncServer},
    DiffSource, Patch, ReqRst, SyncMsg,
};

pub struct Reptra {
    pub serve_addr: Option<ServeAddr>,
    pub service_handle: Option<ServiceHandle>,
}

impl Reptra {
    pub fn new() -> Self {
        Self {
            serve_addr: None,
            service_handle: None,
        }
    }

    pub async fn start_service(&mut self, id: i32) {
        let (serve_addr, incoming) = get_listener().await;
        let mut replica = Replica::new(id);
        replica.init_file_trees().await.unwrap();
        replica.tree(false).await;
        let peer_server = PeerServer {
            replica: Arc::new(replica),
            channels: Arc::new(RwLock::new(HashMap::new())),
        };
        let service_handle = tokio::spawn(async {
            Server::builder()
                .add_service(RsyncServer::new(peer_server))
                .serve_with_incoming(incoming)
                .await
        });
        self.serve_addr = Some(serve_addr);
        self.service_handle = Some(service_handle);
    }

    pub async fn send_port(&self, centra_addr: &ServeAddr) -> MyResult<()> {
        let channel = channel_connect(centra_addr).await?;
        let mut port_sender = PortCollectClient::new(channel);
        let msg = PortNumber {
            port: self.serve_addr.unwrap().port() as i32,
        };
        port_sender
            .send_port(Request::new(msg))
            .await
            .or(Err("failed to send port"))?;
        Ok(())
    }
}

pub async fn reptra_greet_test(id: i32, centra_addr: &ServeAddr) -> MyResult<()> {
    let channel = channel_connect(centra_addr).await?;
    let mut client = GreeterClient::new(channel);
    for i in 0..3 {
        let request = Request::new(HelloRequest {
            name: format!("Say hi {} times from Reptra {}", i, id),
        });
        let response = client.say_hello(request).await;
        let response_msg = response.unwrap().into_inner().message;
        println!("Response from Centra : {}", response_msg);
    }
    info!("greet test passed");
    Ok(())
}
