pub mod booter;
pub mod rsync;

pub mod peer {
    #![allow(non_snake_case)]
    include!("../protos/peer.rs");
}

use crate::{
    centra::{GreeterClient, HelloRequest, PortCollectClient, PortNumber},
    config::{RpcChannel, ServiceHandle},
    info, MyResult,
};
use std::collections::HashMap;

use tonic::{transport::Server, Request};

pub use peer::{
    rsync_client::RsyncClient,
    rsync_server::{Rsync, RsyncServer},
    DiffSource, Patch, ReqRst, SyncMsg,
};

use self::booter::{get_incoming, PeerServer};

pub struct Reptra {
    pub port: Option<u16>,
    pub channels: HashMap<u16, RpcChannel>,
    pub service_handle: Option<ServiceHandle>,
}

impl Reptra {
    pub fn new() -> Self {
        Self {
            port: None,
            channels: HashMap::new(),
            service_handle: None,
        }
    }

    pub async fn get_channel(&mut self, port: u16) -> MyResult<RpcChannel> {
        if self.channels.get(&port).is_some() {
            return Ok(self.channels.get(&port).unwrap().clone());
        }
        let addr = format!("http://[::]:{}", port);
        let channel = RpcChannel::from_shared(addr).or(Err("failed to build channel"))?;
        let channel = channel
            .connect()
            .await
            .or(Err("failed to connect channel"))?;
        self.channels.insert(port, channel.clone());
        return Ok(channel);
    }

    pub async fn start_service(&mut self) {
        let (port, incoming) = get_incoming().await;
        self.port = Some(port);
        let peer_server = PeerServer {};
        let service_handle = tokio::spawn(async {
            Server::builder()
                .add_service(RsyncServer::new(peer_server))
                .serve_with_incoming(incoming)
                .await
        });
        self.service_handle = Some(service_handle);
    }

    pub async fn send_port(&mut self, centra_port: u16) -> MyResult<()> {
        let channel = self.get_channel(centra_port).await?;
        let mut port_sender = PortCollectClient::new(channel);
        let msg = PortNumber {
            port: self.port.unwrap() as i32,
        };
        port_sender
            .send_port(Request::new(msg))
            .await
            .or(Err("failed to send port"))?;
        Ok(())
    }

    pub async fn greet(&mut self, centra_port: u16) -> MyResult<()> {
        let channel = self.get_channel(centra_port).await?;
        let mut client = GreeterClient::new(channel);
        for i in 0..3 {
            let request = Request::new(HelloRequest {
                name: format!("Hi {} times", i),
            });
            let response = client.say_hello(request).await;
            let response_msg = response.unwrap().into_inner().message;
            println!("Response from Centra : {}", response_msg);
        }
        info!("greet test passed");
        Ok(())
    }
}
