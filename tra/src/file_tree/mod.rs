mod node;

pub use node::Node;
pub use std::fs;
pub use std::io;
pub use walkdir::WalkDir;

struct FileTree {
    pub root: Node,
}

impl FileTree {
    pub fn new_from_path(path: &str) -> Self {
        todo!()
    }
    pub fn insert(&mut self) -> io::Result<()> {
        todo!()
    }
}

pub fn test(test_path: &str) -> io::Result<()> {
    for file in WalkDir::new(test_path).into_iter().filter_map(|e| e.ok()) {
        let mut ancestors = file.path().ancestors();
        while let Some(ancestor) = ancestors.next() {
            println!("{}", ancestor.display());
        }
    }
    Ok(())
}
