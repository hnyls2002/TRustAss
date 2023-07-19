use config::BASE_REP_NUM;
use replica::{file_tree, file_watcher};

use crate::reptra::rsync;

pub mod centra;
pub mod config;
pub mod debugger;
pub mod replica;
pub mod reptra;
pub mod timestamp;

async fn demo() {
    rsync::demo();
    file_tree::init(&"demo".to_string()).unwrap();
    file_watcher::file_watch_test(&"demo".to_string());
}

#[tokio::main]
async fn main() {
    let demo_handle = tokio::spawn(async { demo().await });

    // start the the tra algorithm here
    let handle = tokio::spawn(centra::start_centra(BASE_REP_NUM));

    reptra::start_reptra(BASE_REP_NUM).expect("Failed to start reptra");

    demo_handle.await.expect("Failed to join demo thread");

    handle
        .await
        .expect("Failed to join tra thread")
        .expect("Failed to start tra");
}
