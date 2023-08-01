pub mod centra;
pub mod config;
pub mod debugger;
pub mod replica;
pub mod reptra;

use centra::Centra;
use config::{BASE_REP_NUM, TRA_PORT};
use reptra::{rsync, Reptra};

pub use config::MyResult;

async fn demo() {
    rsync::demo();
    let mut rep = replica::Replica::new(TRA_PORT);
    rep.init_file_trees().await.expect("Failed to init replica");
    rep.tree().await;
    rep.watching().await;
}

#[tokio::main]
async fn main() {
    let mut centra = Centra::new(TRA_PORT);
    centra.start_services().await;

    let mut reptra_list: Vec<Reptra> = Vec::new();
    for _ in 0..BASE_REP_NUM {
        let mut reptra = Reptra::new();
        reptra.start_service().await;
        reptra.send_port(TRA_PORT).await.unwrap();
        reptra.greet(TRA_PORT).await.unwrap();
        reptra_list.push(reptra);
    }

    centra.collect_ports(BASE_REP_NUM).await;
}
