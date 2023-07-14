use tokio::signal;

pub const TMP_PATH: &str = "/home/hnyls2002/Desktop/TRustAss/tmp/";

pub const BASE_MAC_NUM: usize = 3;

pub const CHANNEL_BUFFER_SIZE: usize = 1024;

pub async fn ctrl_c_singal() {
    signal::ctrl_c().await.unwrap()
}
