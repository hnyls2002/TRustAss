use self::header::InfoType;

pub mod header;

pub fn first_msg(msg: &str) -> Vec<u8> {
    ("YLS".to_string() + msg).into_bytes()
}

pub struct MsgReader {
    pub recvd: Vec<u8>,
    pub tot_len: usize,
}

impl MsgReader {
    pub fn read_one() -> (InfoType, Vec<u8>) {
        todo!()
    }
}
