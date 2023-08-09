pub mod peer_server;

pub mod peer {
    #![allow(non_snake_case)]
    include!("../protos/peer.rs");
}

use crate::{
    centra::{GreeterClient, HelloRequest, PortCollectClient, PortNumber},
    config::{ServiceHandle, CHANNEL_BUFFER_SIZE},
    info,
    machine::{channel_connect, get_listener, ServeAddr},
    replica::{file_watcher::FileWatcher, Replica},
    MyResult,
};

use inotify::EventMask;
use peer_server::PeerServer;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tonic::{transport::Server, Request};

pub use peer::{
    rsync_client::RsyncClient,
    rsync_server::{Rsync, RsyncServer},
    BoolResult, FetchPatchReq, Patch, QueryReq, QueryRes, SyncReq,
};

pub struct Reptra {
    pub id: i32,
    pub serve_addr: ServeAddr,
    pub service_handle: ServiceHandle,
    pub replica: Arc<Replica>,
    pub file_watcher: FileWatcher,
}

impl Reptra {
    pub async fn new_start_service(id: i32) -> MyResult<Self> {
        let (serve_addr, incoming) = get_listener().await?;
        let file_watcher = FileWatcher::new();
        let watch = file_watcher.get_ifc();
        let replica = Arc::new(Replica::new(id, watch).await);
        replica.init_all().await?;
        replica.tree(false).await;
        let peer_server = PeerServer {
            replica: replica.clone(),
            channels: Arc::new(RwLock::new(HashMap::new())),
        };
        let service_handle = tokio::spawn(async {
            Server::builder()
                .add_service(RsyncServer::new(peer_server))
                .serve_with_incoming(incoming)
                .await
        });
        Ok(Self {
            id,
            serve_addr,
            service_handle,
            replica,
            file_watcher,
        })
    }

    pub async fn send_port(&self, centra_addr: &ServeAddr) -> MyResult<()> {
        let channel = channel_connect(centra_addr).await?;
        let mut port_sender = PortCollectClient::new(channel);
        let msg = PortNumber {
            id: self.id,
            port: self.serve_addr.port() as i32,
        };
        port_sender
            .send_port(Request::new(msg))
            .await
            .or(Err("failed to send port"))?;
        Ok(())
    }

    pub async fn watching(&mut self) -> ! {
        let mut buffer = [0; CHANNEL_BUFFER_SIZE];
        loop {
            let events = self
                .file_watcher
                .inotify
                .read_events_blocking(buffer.as_mut())
                .unwrap();
            for event in events {
                if event.mask != EventMask::IGNORED
                    && !self.file_watcher.is_freezed(&event.wd).await
                {
                    self.file_watcher.display_event(&event).await;
                    self.replica.handle_event(&event).await.unwrap();
                }
            }
            self.replica.tree(true).await;
        }
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
