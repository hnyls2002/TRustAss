pub mod node;

pub use node::Node;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{TMP_PATH, TRA_PORT};
use crate::info;
use crate::replica::checker::check_legal;

pub use std::io::Result as IoResult;

struct FileTree {
    pub prefix: PathBuf,
    pub root: Node,
}

impl FileTree {
    pub fn full_path(&self, relative: &PathBuf) -> IoResult<PathBuf> {
        let mut ret = self.prefix.clone();
        ret.push(relative);
        ret.canonicalize()
    }

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

    pub fn new_from_exist(replica_path: &PathBuf, root_path: &PathBuf) -> Self {
        let mut file_tree = Self {
            prefix: replica_path.clone(),
            root: Node {
                path: Box::new(root_path.clone()),
                is_dir: root_path.is_dir(),
                file_name: root_path.file_name().unwrap().to_str().unwrap().to_string(),
                children: Vec::new(),
            },
        };

        let absolute_path = file_tree
            .full_path(root_path)
            .expect("cat replica path with directory path wrong");

        let files = WalkDir::new(absolute_path).into_iter();
        for file in files.filter_map(|e| e.ok()) {
            let path = file
                .into_path()
                .strip_prefix(replica_path)
                .expect("strip prefix wrong")
                .to_path_buf();
            file_tree.insert(path).expect("New file tree build fails");
        }
        file_tree
    }

    pub fn insert(&mut self, path: PathBuf) -> io::Result<()> {
        // os checks whether the path is a directory
        let is_dir = self.full_path(&path).unwrap().is_dir();
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
                    is_dir,
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

pub fn init(path_str: &String) -> io::Result<()> {
    check_legal(path_str)?;
    info!("Ok, \"{}\" is a legal path", path_str);

    let root_path = Path::new(path_str).to_path_buf();
    let replica_path = Path::new(&format!("{}{}", TMP_PATH, TRA_PORT)).to_path_buf();
    let mut file_tree = FileTree::new_from_exist(&replica_path, &root_path);
    file_tree.organize();
    file_tree.tree();

    Ok(())
}
