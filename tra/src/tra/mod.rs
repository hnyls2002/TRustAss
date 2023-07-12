use std::{thread, usize};

use std::io::Error as IoError;
use std::io::Result as IoResult;

use tokio::net::{TcpListener, TcpStream};

use crate::{
    config::{LOCAL_IP, TRA_PORT},
    info, machine,
};

type MacThread = thread::JoinHandle<Result<(), IoError>>;

pub struct MacInfo {
    pub thread_handle: MacThread,
    pub port: u16,
}

pub async fn handle_connection(mut stream: TcpStream) {
    todo!()
}

pub async fn start_tra(mac_num: usize) -> IoResult<()> {
    let tra_addr = format!("{}:{}", LOCAL_IP, TRA_PORT);
    let listener = TcpListener::bind(tra_addr)
        .await
        .expect("Listener failed to bind");

    // sleep for a second to give the server time to start
    thread::sleep(std::time::Duration::from_millis(200));

    let mut mac_list = Vec::new();
    let mut handle_fibers = Vec::new();

    for _ in 0..mac_num {
        // start a new machine using a thread
        let thread = thread::spawn(machine::start_machine);

        let (stream, peer_addr) = listener
            .accept()
            .await
            .expect("Failed to accept a new connection");

        info!("New connection received");
        info!("Peer address: {}", peer_addr);

        mac_list.push(MacInfo {
            thread_handle: thread,
            port: peer_addr.port(),
        });

        handle_fibers.push(tokio::spawn(async {
            handle_connection(stream).await;
        }));
    }

    for fiber in handle_fibers {
        fiber.await?;
    }

    for mac_info in mac_list {
        mac_info.thread_handle.join().unwrap()?;
    }

    Ok(())
}
