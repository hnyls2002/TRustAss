use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use crate::info;

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
