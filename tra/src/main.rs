pub mod centra;
pub mod checker;
pub mod config;
pub mod conflicts;
pub mod debugger;
pub mod machine;
pub mod replica;
pub mod reptra;
pub mod timestamp;

use centra::Centra;
use checker::check_legal;
use config::{BASE_REP_NUM, TRA_PORT};
use machine::{channel_connect, ServeAddr};
use reptra::{reptra_greet_test, Reptra, RsyncClient};

pub use config::MyResult;
use rustyline::error::ReadlineError;
use tonic::Request;

use crate::reptra::SyncReq;

#[tokio::main]
async fn main() {
    let mut centra = Centra::new(&ServeAddr::new(TRA_PORT));
    centra.start_services().await;

    for id in 1..=BASE_REP_NUM {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let reptra = Reptra::new_start_service(id as i32)
                    .await
                    .expect("failed to start");
                reptra.send_port(&ServeAddr::new(TRA_PORT)).await.unwrap();
                reptra_greet_test(id as i32, &ServeAddr::new(TRA_PORT))
                    .await
                    .unwrap();
                reptra.watching().await;
            });
        });
    }

    centra.collect_ports(BASE_REP_NUM).await;

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("\x1b[34m(tra) ❯ \x1b[0m");
        match readline {
            Ok(line) => {
                let args = line.trim().split_whitespace().collect::<Vec<&str>>();
                if args.len() == 4 && args[0] == "sync" {
                    let id1: i32 = args[1].parse().unwrap();
                    let id2: i32 = args[2].parse().unwrap();
                    let path_rel = args[3].to_string();
                    if id1 as usize <= BASE_REP_NUM
                        && id2 as usize <= BASE_REP_NUM
                        && check_legal(&path_rel)
                    {
                        let addr2 = centra.get_addr(id2);
                        let channel = channel_connect(&addr2).await.unwrap();
                        let mut client = RsyncClient::new(channel);
                        let request = Request::new(SyncReq {
                            port: centra.get_addr(id1).port() as i32,
                            path_rel: path_rel.clone(),
                            is_dir: false,
                        });
                        info!(
                            "Sync Request : replica{}({}) -> replica{}({}), path = \"{}\"",
                            id1,
                            centra.get_addr(id1).port(),
                            id2,
                            centra.get_addr(id2).port(),
                            path_rel
                        );
                        client.request_sync(request).await.unwrap();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                info!("Shutting down the command line interface ...");
                break;
            }
            Err(_) => panic!("Invalid input"),
        }
    }
}
