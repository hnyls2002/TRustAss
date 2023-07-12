use std::io::Result as IoResult;

use tokio::{net::TcpStream, runtime::Runtime};

use crate::{config::{LOCAL_IP, TRA_PORT}, debug};

pub async fn async_work() -> IoResult<()> {
    let addr = format!("{}:{}", LOCAL_IP, TRA_PORT);
    let mut client_socket = TcpStream::connect(addr).await?;
    todo!()
}

pub fn start_machine() -> IoResult<()> {
    let rt = Runtime::new()?;
    rt.block_on(async_work())?;
    Ok(())
}
