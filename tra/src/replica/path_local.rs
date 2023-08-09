use std::path::{Path, PathBuf};

use super::node::NodeStatus;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathLocal {
    prefix: PathBuf,
    full_path: PathBuf,
}

impl AsRef<Path> for PathLocal {
    fn as_ref(&self) -> &Path {
        &self.full_path
    }
}

impl PathLocal {
    pub fn is_dir(&self) -> bool {
        self.full_path.is_dir()
    }

    pub fn exists(&self) -> bool {
        self.full_path.exists()
    }

    pub fn status(&self) -> NodeStatus {
        self.full_path
            .exists()
            .then(|| NodeStatus::Exist)
            .unwrap_or(NodeStatus::Deleted)
    }

    pub fn prefix(&self) -> &PathBuf {
        &self.prefix
    }

    pub fn display(&self) -> String {
        self.full_path.to_str().unwrap().to_string()
    }

    pub fn canonicalize(&self) -> Option<Self> {
        self.full_path.canonicalize().ok().map(|full_path| Self {
            prefix: self.prefix.clone(),
            full_path,
        })
    }

    pub fn file_name(&self) -> Option<String> {
        if self.full_path == self.prefix {
            None
        } else {
            Some(
                self.full_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
        }
    }

    pub fn pop(&mut self) -> Option<String> {
        self.file_name().is_some().then(|| {
            let name = self.file_name().unwrap();
            self.full_path.pop();
            name
        })
    }

    pub fn get_walk(&self) -> Vec<String> {
        let mut tmp = self.clone();
        let mut ret: Vec<String> = Vec::new();
        while let Some(name) = tmp.pop() {
            ret.push(name)
        }
        ret
    }

    pub fn to_rel(&self) -> String {
        self.full_path
            .strip_prefix(&self.prefix)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn new_from_rel(prefix: impl AsRef<Path>, path: impl AsRef<Path>) -> Self {
        assert!(path.as_ref().is_relative());
        let full_path = prefix.as_ref().join(path);
        Self {
            prefix: prefix.as_ref().to_path_buf(),
            full_path,
        }
    }

    pub fn new_from_local(prefix: impl AsRef<Path>, path: impl AsRef<Path>) -> Self {
        assert!(path.as_ref().is_absolute());
        Self {
            prefix: prefix.as_ref().to_path_buf(),
            full_path: path.as_ref().to_path_buf(),
        }
    }

    pub fn join_name(&self, name: impl AsRef<Path>) -> Self {
        Self {
            prefix: self.prefix.clone(),
            full_path: self.full_path.join(name),
        }
    }
}
