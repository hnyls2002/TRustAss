use std::thread;

use config::TMP_PATH;

pub mod client;
pub mod config;
pub mod debugger;
pub mod file_tree;
pub mod file_watcher;
pub mod machine;
pub mod protos;
pub mod rsync;
pub mod server;
pub mod timestamp;

pub fn test_socket() -> std::io::Result<()> {
    let server_thread = thread::spawn(|| {
        server::start_server().expect("Server failed");
    });

    // sleep for a second to give the server time to start
    thread::sleep(std::time::Duration::from_secs(1));

    let client_thread = thread::spawn(|| {
        client::start_client().expect("Client failed");
    });

    server_thread.join().expect("Server thread panicked");
    client_thread.join().expect("Client thread panicked");

    Ok(())
}

async fn test1() {
    println!("This is test1");
}

async fn test2() {
    println!("This is test2");
}

#[tokio::main]
async fn main() {
    if false {
        test_socket().unwrap();
        rsync::demo();
        let folder_path_str = TMP_PATH.to_string() + "folder/";
        file_tree::init(&folder_path_str).unwrap();
        file_watcher::file_watch_test(&folder_path_str);
        rsync::rsync().unwrap();
    }

    let future1 = test1();
    let future2 = test2();

    future1.await;
    future2.await;
}
