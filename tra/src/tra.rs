use std::{thread, usize};

use std::io::Error as IoError;
use std::io::Result as IoResult;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::{
    config::{LOCAL_IP, TRA_PORT},
    info, machine,
};

pub struct MacInfo {
    pub thread_handle: thread::JoinHandle<Result<(), IoError>>,
    pub port: u16,
}

pub async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(n) => {
                if n != 0 {
                    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
                    let response = "Hello from server";
                    stream
                        .write(response.as_bytes())
                        .await
                        .expect("Could not write");
                } else {
                    // only something like EOF would cause n to be 0
                    info!("Connection closed");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                return;
            }
        }
    }
}

pub async fn start_tra(mac_num: usize) -> IoResult<()> {
    let tra_addr = format!("{}:{}", LOCAL_IP, TRA_PORT);
    let listener = TcpListener::bind(tra_addr)
        .await
        .expect("Listener failed to bind");

    // sleep for a second to give the server time to start
    thread::sleep(std::time::Duration::from_secs(1));

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
