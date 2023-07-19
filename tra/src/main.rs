use config::BASE_REP_NUM;

use crate::reptra::rsync;

pub mod centra;
pub mod config;
pub mod debugger;
pub mod file_sync;
pub mod file_tree;
pub mod file_watcher;
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

    demo_handle.await.expect("Failed to join demo thread");

    // start the the tra algorithm here
    let handle = tokio::spawn(centra::start_centra(BASE_REP_NUM));

    reptra::start_reptra(BASE_REP_NUM).expect("Failed to start reptra");

    handle
        .await
        .expect("Failed to join tra thread")
        .expect("Failed to start tra");
}
