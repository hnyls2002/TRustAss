use self::service::boot_server;
use crate::{
    centra::{GreeterClient, HelloRequest},
    debug,
};
use std::{io::Result as IoResult, thread};
use tokio::runtime::Runtime;
use tonic::{transport::Channel, Request};

pub mod service;
pub async fn async_work() -> IoResult<()> {
    let channel = Channel::from_static("http://[::]:8080")
        .connect()
        .await
        .unwrap();

    // boot the machine server here
    let server = boot_server(channel.clone()).await;

    // ----------------- do machine things below -----------------
    let mut client = GreeterClient::new(channel);
    let mut counter = 0;

    loop {
        let request = Request::new(HelloRequest {
            name: format!("asd {} times", counter),
        });

        counter += 1;

        let response = client.say_hello(request).await;

        let response_msg = response.unwrap().into_inner().message;

        debug!("{}", response_msg);

        if counter >= 3 {
            break;
        }
    }

    server.await?.expect("server failed");

    Ok(())
}

pub fn start_machine(mac_num: usize) -> IoResult<()> {
    let mut mac_threads = Vec::new();
    for _ in 0..mac_num {
        mac_threads.push(thread::spawn(|| -> IoResult<()> {
            let rt = Runtime::new()?;

            // use this to enter the runtime context, so that we can spawn tasks
            // that is, calling `boot_server()` here would not cause panic
            // let _guard = rt.enter();
            // let (server, port) = boot_server();
            // info!("the port number is {}", port);

            rt.block_on(async_work())?;
            Ok(())
        }));
    }

    for thread in mac_threads {
        thread.join().expect("mac thread join failed")?;
    }

    Ok(())
}
