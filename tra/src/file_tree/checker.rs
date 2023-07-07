use std::{io, path::PathBuf};

use regex::Regex;

// 1. start with /
// 2. seperated by /
// 3. only contains [a-zA-Z0-9_.-]
// 4. end with / or nothing
const LEAGL_RULE: &'static str = r#"^(/([a-zA-Z0-9_.-]+))+/?$"#;

pub fn check_legal(path_str: &String) -> io::Result<()> {
    let regex = Regex::new(LEAGL_RULE).unwrap();
    if regex.is_match(path_str) {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("illegal path: {}", path_str),
    ))
}

// decomp a path to a vector of string
// use relative path to root
pub fn decomp(mut path: PathBuf, root: &PathBuf) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    while path != *root {
        ret.push(path.file_name().unwrap().to_str().unwrap().to_string());
        path.pop();
    }
    ret
}
