use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use crate::{info, machine};

pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n != 0 {
                    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
                    let response = "Hello from server";
                    stream.write(response.as_bytes()).expect("Could not write");
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

pub fn start_server() -> std::io::Result<()> {
    let addr = "127.0.0.1:1235";
    let listener = TcpListener::bind(addr)?;
    let mut thread_vec = Vec::new();

    // read incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection received");
                thread_vec.push(thread::spawn(|| {
                    handle_connection(stream);
                }));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    for thread in thread_vec {
        thread.join().expect("Could not join thread");
    }

    Ok(())
}

pub fn test_socket() -> std::io::Result<()> {
    let server_thread = thread::spawn(|| {
        start_server().expect("Server failed");
    });

    // sleep for a second to give the server time to start
    thread::sleep(std::time::Duration::from_secs(1));

    let client_thread = thread::spawn(|| {
        machine::start_client().expect("Client failed");
    });

    server_thread.join().expect("Server thread panicked");
    client_thread.join().expect("Client thread panicked");

    Ok(())
}
