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
}

#[tokio::main]
async fn main() {
    let mut centra = Centra::new(&ServeAddr::new(TRA_PORT));
    centra.start_services().await;

    let mut thread_list = Vec::new();
    for id in 1..=BASE_REP_NUM {
        thread_list.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut reptra = Reptra::new_start_service(id as i32).await;
                reptra.send_port(&ServeAddr::new(TRA_PORT)).await.unwrap();
                reptra_greet_test(id as i32, &ServeAddr::new(TRA_PORT))
                    .await
                    .unwrap();
                reptra.watching().await;
            });
        }));
    }

    centra.collect_ports(BASE_REP_NUM).await;

    for t in thread_list {
        t.join().unwrap();
    }
}
