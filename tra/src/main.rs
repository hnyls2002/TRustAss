pub mod banner;
pub mod centra;
pub mod checker;
pub mod config;
pub mod conflicts;
pub mod debugger;
pub mod machine;
pub mod replica;
pub mod reptra;
pub mod timestamp;

use banner::{BannerOut, SyncBanner};
use centra::Centra;
use checker::check_legal;
use config::{BASE_REP_NUM, TRA_PORT};
use machine::{channel_connect, ServeAddr};
use reptra::{Reptra, RsyncClient};

pub use config::MyResult;
use rustyline::error::ReadlineError;
use tonic::Request;

use crate::reptra::{SyncReq, Void};

async fn sync_command(args: &Vec<&str>, centra: &Centra) -> MyResult<()> {
    let id1: i32 = args.get(1).ok_or("")?.parse().or(Err(""))?;
    let id2: i32 = args.get(2).ok_or("")?.parse().or(Err(""))?;
    let path_rel = args.get(3).ok_or("")?.to_string();
    if id1 as usize <= BASE_REP_NUM
        && id2 as usize <= BASE_REP_NUM
        && id1 != id2
        && check_legal(&path_rel)
    {
        let addr2 = centra.get_addr(id2);
        let channel = channel_connect(&addr2).await.unwrap();
        let mut client = RsyncClient::new(channel);
        let request = Request::new(SyncReq {
            port: centra.get_addr(id1).port() as i32,
            path_rel: path_rel.clone(),
        });
        SyncBanner::sync_request(
            id1,
            centra.get_addr(id1).port(),
            id2,
            centra.get_addr(id2).port(),
            path_rel,
        );
        client.request_sync(request).await.unwrap();
    } else {
        return Err("".into());
    }
    Ok(())
}

async fn tree_command(args: &Vec<&str>, centra: &Centra) -> MyResult<()> {
    let id: i32 = args.get(1).ok_or("")?.parse().or(Err(""))?;
    if id as usize <= BASE_REP_NUM {
        let addr = centra.get_addr(id);
        let channel = channel_connect(&addr).await.unwrap();
        let mut client = RsyncClient::new(channel);
        client.tree(Request::new(Void {})).await.unwrap();
    } else {
        return Err("".into());
    }
    Ok(())
}

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
                // reptra_greet_test(id as i32, &ServeAddr::new(TRA_PORT)).await.unwrap();
                reptra.watching().await;
            });
        });
    }

    centra.collect_ports(BASE_REP_NUM).await;

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("\x1b[1m(tra) ❯ \x1b[0m");
        match readline {
            Ok(line) => {
                let args = line.trim().split_whitespace().collect::<Vec<&str>>();
                if !args.is_empty() {
                    if args[0] == "sync" {
                        if sync_command(&args, &centra).await.is_err() {
                            BannerOut::cross("Invalid input");
                        }
                    } else if args[0] == "tree" {
                        if tree_command(&args, &centra).await.is_err() {
                            BannerOut::cross("Invalid input");
                        }
                    } else {
                        BannerOut::cross("Invalid input");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Shutting down the command line interface ...");
                break;
            }
            Err(_) => panic!("Invalid input"),
        }
    }
}
