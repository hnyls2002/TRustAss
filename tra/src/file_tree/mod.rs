pub mod checker;
mod node;

pub use node::Node;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::file_tree::checker::{check_legal, decomp};
use crate::info;

struct FileTree {
    pub root: Node,
}

impl FileTree {
    pub fn new(root_path: &PathBuf) -> Self {
        let root = Node {
            path: Box::new(root_path.clone()),
            file_name: root_path.file_name().unwrap().to_str().unwrap().to_string(),
            children: Vec::new(),
        };
        let mut file_tree = Self { root };

        let files = WalkDir::new(root_path).into_iter();
        for file in files.filter_map(|e| e.ok()) {
            let path = file.into_path();
            let walk = decomp(path.clone(), root_path);
            file_tree
                .insert(walk, path)
                .expect("New file tree build fails");
        }
        file_tree
    }

    pub fn insert(&mut self, walk: Vec<String>, path: PathBuf) -> io::Result<()> {
        self.root.insert(walk, path)
    }

    // just like the tree command
    pub fn tree(&self) {
        self.root.tree(Vec::new());
    }

    // sort each level's file name by dictionary order
    pub fn organize(&mut self) {
        self.root.organize();
    }
}

pub fn init(path_str: &str) -> io::Result<()> {
    check_legal(path_str)?;
    info!("Ok, \"{}\" is a legal path", path_str);

    let path = Path::new(path_str).to_path_buf();
    let mut file_tree = FileTree::new(&path);
    file_tree.organize();
    file_tree.tree();

    Ok(())
}
