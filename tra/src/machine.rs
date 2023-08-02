use rand::Rng;
use std::net::TcpListener as StdListener;
use tonic::transport::server::TcpIncoming;

use crate::{config::RpcChannel, MyResult};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ServeAddr {
    port: u16,
}

impl ServeAddr {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn addr(&self) -> String {
        format!("[::]:{}", self.port)
    }

    pub fn http_addr(&self) -> String {
        format!("http://{}", self.addr())
    }
}

pub async fn get_listener() -> (ServeAddr, TcpIncoming) {
    let mut rng = rand::thread_rng();
    loop {
        let port = rng.gen_range(49152..=65535);
        let listener = StdListener::bind(format!("[::]:{}", port));
        if let Ok(listener) = listener {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
            break (ServeAddr::new(port), incoming);
        }
    }
}

pub async fn channel_connect(serve_addr: &ServeAddr) -> MyResult<RpcChannel> {
    let addr = serve_addr.http_addr();
    let channel = RpcChannel::from_shared(addr).or(Err("failed to build channel"))?;
    let channel = channel
        .connect()
        .await
        .or(Err("failed to connect channel"))?;
    Ok(channel)
}
