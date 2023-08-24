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
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};
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
    pub file_watcher: RefCell<FileWatcher>,
}

impl Reptra {
    pub async fn new_start_service(id: i32) -> MyResult<Self> {
        let (serve_addr, incoming) = get_listener().await?;
        let file_watcher = FileWatcher::new();
        let watch = file_watcher.get_ifc();
        let c_lock = Arc::new(Mutex::new(()));
        let replica = Arc::new(Replica::new(id, watch, c_lock).await);
        replica.init_all().await?;
        // replica.tree(false).await;
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
            file_watcher: RefCell::new(file_watcher),
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
            .map_err(|e| "failed to send port : ".to_string() + &e.to_string())?;
        Ok(())
    }

    pub async fn watching(&self) -> ! {
        let mut buffer = [0; CHANNEL_BUFFER_SIZE];
        loop {
            let events = self
                .file_watcher
                .borrow_mut()
                .inotify
                .read_events_blocking(buffer.as_mut())
                .unwrap();
            for event in events {
                if event.mask != EventMask::IGNORED
                    && !self.file_watcher.borrow().is_freezed(&event.wd).await
                {
                    self.file_watcher.borrow().display_event(&event).await;
                    self.replica.handle_event(&event).await.unwrap();
                    self.replica.tree(true).await;
                }
            }
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
