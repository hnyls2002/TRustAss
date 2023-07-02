use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
};

pub fn start_client() -> std::io::Result<()> {
    let addr = "127.0.0.1:1235";
    let mut counter = 0;
    let mut client_socket = TcpStream::connect(addr)?;

    loop {
        let message = format!("Hello from client {} times", counter);
        counter += 1;

        client_socket.write(message.as_bytes())?;

        let mut response = [0; 1024];
        client_socket.read(&mut response)?;

        println!("Response: {}", String::from_utf8_lossy(&response[..]));

        thread::sleep(std::time::Duration::from_secs_f64(0.1));
    }
}
