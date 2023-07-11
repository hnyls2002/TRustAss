use std::{io::Result as IoResult, thread};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    runtime::Runtime,
};

use crate::config::{LOCAL_IP, TRA_PORT};

fn first_msg(msg: &str) -> Vec<u8> {
    ("YLS".to_string() + msg).into_bytes()
}

pub async fn async_work() -> IoResult<()> {
    let addr = format!("{}:{}", LOCAL_IP, TRA_PORT);
    let mut counter = 0;
    let mut client_socket = TcpStream::connect(addr).await?;

    loop {
        let message = first_msg(format!("Hello from client {} times", counter).as_str());
        counter += 1;

        client_socket.write(message.as_slice()).await?;

        client_socket.write(message.as_slice()).await?;

        let mut response = [0; 1024];
        client_socket.read(&mut response).await?;

        println!("Response: {}", String::from_utf8_lossy(&response[..]));

        thread::sleep(std::time::Duration::from_secs_f64(0.1));

        break;
    }

    // must manually shutdown the socket to close the connection
    client_socket.shutdown().await?;

    Ok(())
}

pub fn start_machine() -> IoResult<()> {
    let rt = Runtime::new()?;
    rt.block_on(async_work())?;
    Ok(())
}
