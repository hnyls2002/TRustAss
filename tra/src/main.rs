use std::thread;

use inotify::{Inotify, WatchMask};

mod client;
mod debugger;
mod file_tree;
mod server;

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

pub fn file_watch_test(dir_path: &str) {
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

    // file_watch_test("tmp/");

    file_tree::init("/home/hnyls2002/Desktop/TRustAss/tmp/folder").unwrap();
}
