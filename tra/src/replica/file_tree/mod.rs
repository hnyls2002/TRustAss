pub mod node;

use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;

pub use node::Node;
pub use std::io::Result as IoResult;

use super::RepMeta;

pub struct FileTree {
    pub rep_meta: Arc<RepMeta>,
    pub root: Node,
}

impl FileTree {
    // decomp a path to a vector of string
    pub fn decompose(path: &PathBuf) -> Vec<String> {
        let mut tmp_path = path.clone();
        let mut ret: Vec<String> = Vec::new();
        while tmp_path.file_name().is_some() {
            ret.push(tmp_path.file_name().unwrap().to_str().unwrap().to_string());
            tmp_path.pop();
        }
        ret
    }

    pub fn new_from_exist(rep_meta: Arc<RepMeta>, root_path: &PathBuf) -> Self {
        let mut file_tree = Self {
            rep_meta: rep_meta.clone(),
            root: Node {
                path: Box::new(root_path.clone()),
                is_dir: root_path.is_dir(),
                file_name: root_path.file_name().unwrap().to_str().unwrap().to_string(),
                children: Vec::new(),
            },
        };

        let absolute_path = file_tree
            .rep_meta
            .to_absolute(root_path)
            .expect("to absolute path fails");

        let files = WalkDir::new(absolute_path).into_iter();
        for file in files.filter_map(|e| e.ok()) {
            let path = rep_meta
                .to_relative(&file.into_path())
                .expect("to relative path fails");
            file_tree.insert(path).expect("New file tree build fails");
        }
        file_tree
    }

    pub fn insert(&mut self, path: PathBuf) -> io::Result<()> {
        let mut walk = Self::decompose(&path);
        let mut current = &mut self.root;
        assert_eq!(current.file_name, walk.pop().unwrap());
        while !walk.is_empty() {
            let entry = walk.pop().unwrap();
            if !current
                .children
                .iter()
                .any(|child| child.file_name == entry)
            {
                current.children.push(Node {
                    path: Box::new(path.clone()),
                    is_dir: self.rep_meta.check_is_dir(&path),
                    file_name: entry.clone(),
                    children: Vec::new(),
                })
            }
            current = current
                .children
                .iter_mut()
                .find(|child| child.file_name == entry)
                .unwrap();
        }
        Ok(())
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
