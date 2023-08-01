use rand::Rng;
use std::net::TcpListener as StdListener;
use tonic::transport::server::TcpIncoming;

pub struct PeerServer {}

pub async fn get_incoming() -> (u16, TcpIncoming) {
    let mut rng = rand::thread_rng();
    loop {
        let port = rng.gen_range(49152..=65535);
        let listener = StdListener::bind(format!("[::]:{}", port));
        if let Ok(listener) = listener {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
            break (port, incoming);
        }
    }
}
