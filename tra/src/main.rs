pub mod centra;
pub mod config;
pub mod debugger;
pub mod replica;
pub mod reptra;

use std::thread;

use config::{BASE_REP_NUM, TRA_PORT};
use replica::file_watcher;
use reptra::rsync;

pub use config::MyResult;

async fn demo() {
    rsync::demo();
    let mut rep = replica::Replica::new(TRA_PORT);
    rep.init_file_trees().await.expect("Failed to init replica");
    rep.tree().await;
    let mut file_watcher = file_watcher::FileWatcher::new_from_replica(&rep).await;
    thread::spawn(move || {
        file_watcher.work();
    });
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
