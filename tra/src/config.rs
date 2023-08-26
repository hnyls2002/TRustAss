use fast_rsync::SignatureOptions;
use lazy_static::lazy_static;
use tokio::signal;

fn get_tmp_path() -> String {
    let mut path_abs = std::env::current_dir().unwrap();
    path_abs.push("tmp/");
    path_abs.to_str().unwrap().to_string()
}

lazy_static! {
    pub static ref TMP_PATH: String = get_tmp_path();
}

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

pub fn sync_folder_prefix(id: i32) -> String {
    format!("{}replica-{}", *TMP_PATH, id)
}
