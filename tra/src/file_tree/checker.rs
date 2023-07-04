use std::io;

use regex::Regex;

// 1. start with /
// 2. seperated by /
// 3. only contains [a-zA-Z0-9_.-]
// 4. end with / or nothing
const LEAGL_RULE: &'static str = r#"^(/([a-zA-Z0-9_.-]+))+/?$"#;

pub fn check_legal(path_str: &str) -> io::Result<()> {
    let regex = Regex::new(LEAGL_RULE).unwrap();
    if regex.is_match(path_str) {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("illegal path: {}", path_str),
    ))
}
