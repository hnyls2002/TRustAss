use std::io;

use regex::Regex;

// 1. start with /
// 2. seperated by /
// 3. only contains [a-zA-Z0-9_.-]
// 4. end with / or nothing
#[allow(dead_code)]
const LEAGL_RULE_ABSOLUTE: &'static str = r#"^(/([a-zA-Z0-9_.-]+))+/?$"#;
#[allow(dead_code)]
const LEGAL_RULE_RELATIVE: &'static str = r#"^([a-zA-Z0-9_.-]+/)*([a-zA-Z0-9_.-]+)?$"#;

pub fn check_legal(path_str: &String) -> io::Result<()> {
    let regex = Regex::new(LEGAL_RULE_RELATIVE).unwrap();
    if regex.is_match(path_str) {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("illegal path: {}", path_str),
    ))
}
