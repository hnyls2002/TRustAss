use std::io;

use fast_rsync::{apply, diff, Signature, SignatureOptions};

pub fn demo() {
    let data1 = "hello fuck".as_bytes();
    let data2 = "fuck you".as_bytes();
    let option = SignatureOptions {
        block_size: 1024,
        crypto_hash_size: 8,
    };
    let sig = Signature::calculate(data1, option);
    let index_sig = sig.index();
    let mut buf: Vec<u8> = Vec::new();
    let mut res: Vec<u8> = Vec::new();
    diff(&index_sig, data2, &mut buf).unwrap();
    apply(data1, &buf, &mut res).unwrap();
    println!("{}", std::str::from_utf8(res.as_slice()).unwrap());
}

pub fn rsync() -> io::Result<()> {
    Ok(())
}
