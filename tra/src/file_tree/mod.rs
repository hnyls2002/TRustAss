pub mod checker;
mod node;

pub use node::{Node, NodeType};
pub use std::fs;
pub use std::io;
use std::path::Path;
pub use walkdir::WalkDir;

use crate::file_tree::checker::check_legal;

struct FileTree {
    pub root: Node,
}

impl FileTree {
    pub fn new(path_str: &str) -> Self {
        for file in WalkDir::new(path_str).into_iter().filter_map(|e| e.ok()) {
            let mut ancestors = file.path().ancestors();
            println!("the file is {:?}", file.path());
            decomp(file.path());
        }
        Self {
            root: Node::new(path_str.to_string(), todo!()),
        }
    }

    pub fn insert(&mut self) -> io::Result<()> {
        todo!()
    }

    // just like the tree command
    pub fn tree(&self) {}
}

pub fn decomp(path: &Path) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    path.ancestors().for_each(|p| {});

    for name in ret.iter() {
        println!("name is {:?}", name);
    }
    ret
}

pub fn test(path_str: &str) -> io::Result<()> {
    loop {
        // input a string
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.truncate(input.len() - 1);
        let path = Path::new(&input);
        println!(
            "is a legal file name : {}",
            check_legal(input.as_str()).map_or(false, |_| true)
        );
    }
    Ok(())
}
