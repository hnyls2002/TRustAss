use inotify::{Inotify, WatchMask};

use crate::debug;

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
