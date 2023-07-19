use tokio::signal;

pub const TMP_PATH: &str = "/home/hnyls2002/Desktop/TRustAss/tmp/";

pub const BASE_REP_NUM: usize = 3;

pub const CHANNEL_BUFFER_SIZE: usize = 1024;

pub const TRA_PORT: u16 = 8080;
pub const TRA_STATIC_ADDR: &str = "http://[::]:8080";

pub async fn ctrl_c_singal() {
    signal::ctrl_c().await.unwrap()
}
