pub mod booter;
pub mod rsync;
pub mod peer {
    include!("../protos/peer.rs");
}

use crate::{
    centra::{GreeterClient, HelloRequest},
    config::{CHANNEL_BUFFER_SIZE, TRA_STATIC_ADDR},
    debug,
};
use booter::boot_server;
use std::{io::Result as IoResult, thread};
use tokio::{runtime::Runtime, sync::mpsc};
use tonic::Request;

pub use peer::{
    rsync_client::RsyncClient,
    rsync_server::{Rsync, RsyncServer},
    DiffSource, Patch, ReqRst, SyncMsg,
};

pub async fn greet_test(tonic_channel: tonic::transport::Channel) -> IoResult<()> {
    let mut client = GreeterClient::new(tonic_channel);
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

    Ok(())
}

pub async fn async_work() -> IoResult<()> {
    // build the tonic channel to connect to centra server
    let tonic_channel = tonic::transport::Channel::from_static(TRA_STATIC_ADDR)
        .connect()
        .await
        .unwrap();

    // build the mpsc channel to dispatch sync tasks in a single reptra
    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    // boot the reptra server here
    let server = boot_server(tonic_channel.clone(), &tx).await;

    // ----------------- do reptra things below -----------------
    let greet_channel = tonic_channel.clone();
    let greet_handle = tokio::spawn(async {
        greet_test(greet_channel).await.expect("greet test failed");
    });

    greet_handle.await?;

    server.await?.expect("server failed");

    Ok(())
}

pub fn start_reptra(rep_num: usize) -> IoResult<()> {
    let mut rep_threads = Vec::new();
    for _ in 0..rep_num {
        rep_threads.push(thread::spawn(|| -> IoResult<()> {
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

    for thread in rep_threads {
        thread.join().expect("peer thread join failed")?;
    }

    Ok(())
}
