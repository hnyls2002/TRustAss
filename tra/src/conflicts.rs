use diff::lines;
use std::str::from_utf8;
use tokio::process::Command;

use crate::{
    banner::BannerOut,
    replica::{
        meta::{get_sync_bytes, read_bytes, write_bytes},
        node::SyncOption,
        path_local::PathLocal,
    },
    MyResult,
};

pub fn format_diff(diffed: Vec<diff::Result<&str>>) -> String {
    let mut last_status = diff::Result::Both("", "");
    let mut tui = String::new();
    for res in diffed {
        match res {
            diff::Result::Both(s, _) => {
                match last_status {
                    diff::Result::Both(_, _) => {}
                    diff::Result::Left(_) => tui.push_str("<<<<<<< LOCAL END\n"),
                    diff::Result::Right(_) => tui.push_str(">>>>>>> REMOTE END\n"),
                }
                tui.push_str(format!("{}\n", s).as_str());
                last_status = diff::Result::Both("", "");
            }
            diff::Result::Left(s) => {
                match last_status {
                    diff::Result::Both(_, _) => tui.push_str("<<<<<<< LOCAL BEGIN\n"),
                    diff::Result::Left(_) => {}
                    diff::Result::Right(_) => {
                        tui.push_str("<<<<<<< REMOTE END\n");
                        tui.push_str(">>>>>>> LOCAL BEGIN\n");
                    }
                }
                tui.push_str(format!("{}\n", s).as_str());
                last_status = diff::Result::Left("");
            }

            diff::Result::Right(s) => {
                match last_status {
                    diff::Result::Both(_, _) => tui.push_str(">>>>>>> REMOTE BEGIN\n"),
                    diff::Result::Left(_) => {
                        tui.push_str(">>>>>>> LOCAL END\n");
                        tui.push_str("<<<<<<< REMOTE BEGIN\n");
                    }
                    diff::Result::Right(_) => {}
                }
                tui.push_str(format!("{}\n", s).as_str());
                last_status = diff::Result::Right("");
            }
        }
    }
    if last_status == diff::Result::Left("") {
        tui.push_str("<<<<<<< LOCAL END\n");
    } else if last_status == diff::Result::Right("") {
        tui.push_str(">>>>>>> REMOTE END\n");
    }
    tui
}

pub async fn manually_resolve(path: &PathLocal, op: SyncOption) -> MyResult<bool> {
    let original = read_bytes(path).await?;
    let synced = get_sync_bytes(path, op.client).await?;
    let diffed = lines(
        from_utf8(&original.as_slice()).unwrap(),
        from_utf8(&synced.as_slice()).unwrap(),
    );
    let tui = format_diff(diffed);
    write_bytes(path, tui).await?;
    let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());
    let edit_command = Command::new(editor)
        .arg(path.as_ref())
        .status()
        .await
        .expect("failed to execute editor");
    if !edit_command.success() {
        // write back the original file if no changes were made
        write_bytes(path, original).await?;
        BannerOut::cross("failed to execute editor");
        return Ok(false);
    }
    Ok(true)
}
