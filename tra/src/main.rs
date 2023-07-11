use config::TMP_PATH;

pub mod config;
pub mod debugger;
pub mod file_tree;
pub mod file_watcher;
pub mod machine;
pub mod message;
pub mod protos;
pub mod rsync;
pub mod timestamp;
pub mod tra;

async fn demo() {
    rsync::demo();
    let folder_path_str = TMP_PATH.to_string() + "folder/";
    file_tree::init(&folder_path_str).unwrap();
    file_watcher::file_watch_test(&folder_path_str);
    rsync::rsync().unwrap();

    async fn test1() {
        println!("This is test1");
    }

    async fn test2() {
        println!("This is test2");
    }

    let future1 = test1();
    let future2 = test2();

    future1.await;
    future2.await;
}

#[tokio::main]
async fn main() {
    if false {
        demo().await;
    }

    // start the tra server
    tra::start_tra(1).await.expect("Failed to start tra");
}
