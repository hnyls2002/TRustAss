use std::thread;

use config::TMP_PATH;
use inotify::{Inotify, WatchMask};

pub mod client;
pub mod config;
pub mod debugger;
pub mod file_tree;
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

pub fn file_watch_test(dir_path: &String) {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    let path = std::path::Path::new(dir_path);
    std::fs::create_dir_all(path).unwrap();

    debug!("All events can be watched {:?}", WatchMask::ALL_EVENTS);

    inotify.watches().add(path, WatchMask::ALL_EVENTS).unwrap();

    let mut buffer = [0; 1024];

    loop {
        debug!("Waiting for events");
        let events = inotify.read_events_blocking(buffer.as_mut()).unwrap();

        for event in events {
            println!("{:?}", event);
        }
    }
}

fn main() {
    // test_socket().unwrap();

    if true {
        rsync::demo();
        let folder_path_str = TMP_PATH.to_string() + "folder/";
        file_tree::init(&folder_path_str).unwrap();
        file_watch_test(&folder_path_str);
    }
}
