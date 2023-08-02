pub mod centra;
pub mod config;
pub mod debugger;
pub mod machine;
pub mod replica;
pub mod reptra;

use centra::Centra;
use config::{BASE_REP_NUM, TRA_PORT};
use machine::ServeAddr;
use reptra::{peer_server, reptra_greet_test, Reptra};

pub use config::MyResult;

async fn demo() {
    peer_server::demo();
    let mut rep = replica::Replica::new(1926);
    rep.init_file_trees().await.expect("Failed to init replica");
    rep.tree(false).await;
    rep.watching().await;
}

#[tokio::main]
async fn main() {
    let mut centra = Centra::new(&ServeAddr::new(TRA_PORT));
    centra.start_services().await;

    let mut thread_list = Vec::new();
    for id in 1..=BASE_REP_NUM {
        thread_list.push(std::thread::spawn(move || {
            let mut reptra = Reptra::new();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                reptra.start_service(id as i32).await;
                reptra.send_port(&ServeAddr::new(TRA_PORT)).await.unwrap();
                reptra_greet_test(id as i32, &ServeAddr::new(TRA_PORT))
                    .await
                    .unwrap();
            });
        }));
    }

    centra.collect_ports(BASE_REP_NUM).await;

    for t in thread_list {
        t.join().unwrap();
    }
}
