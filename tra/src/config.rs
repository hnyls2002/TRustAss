use fast_rsync::SignatureOptions;
use tokio::signal;

pub const TMP_PATH: &str = "/home/hnyls2002/Desktop/TRustAss/tmp/";

pub const BASE_REP_NUM: usize = 3;

pub const CHANNEL_BUFFER_SIZE: usize = 1024;

pub const TRA_PORT: u16 = 8080;
pub const TRA_STATIC_ADDR: &str = "http://[::]:8080";

pub async fn ctrl_c_singal() {
    signal::ctrl_c().await.unwrap()
}

pub const SIG_OPTION: SignatureOptions = SignatureOptions {
    block_size: 1024,
    crypto_hash_size: 16,
};

pub type MyResult<T> = Result<T, String>;
pub type RpcChannel = tonic::transport::Channel;
pub type MpscSender<T> = tokio::sync::mpsc::Sender<T>;
pub type MpscReceiver<T> = tokio::sync::mpsc::Receiver<T>;
pub type ServiceHandle = tokio::task::JoinHandle<Result<(), tonic::transport::Error>>;

#[macro_export]
macro_rules! unwrap_res {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err.to_string()),
        }
    };
}
