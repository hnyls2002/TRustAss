use std::{io, path::PathBuf};

pub struct Node {
    pub path: Box<PathBuf>,
    pub file_name: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn insert(&mut self, mut walk: Vec<String>, path: PathBuf) -> io::Result<()> {
        if let Some(file_name) = walk.pop() {
            // println!("file name is {}", file_name);

            let res = (&mut self.children)
                .into_iter()
                .find(|child| child.file_name == file_name);

            let entry: &mut Node = if let Some(child) = res {
                child
            } else {
                let mut new_path = self.path.clone();
                new_path.push(file_name.clone());
                self.children.push(Node {
                    path: new_path,
                    file_name: file_name.clone(),
                    children: Vec::new(),
                });
                self.children.last_mut().unwrap()
            };

            entry.insert(walk, path)?;
        } else {
            assert_eq!(path, *self.path);
        }
        Ok(())
    }

    pub fn tree(&self, is_last: Vec<bool>) {
        // println!("{}", self.path.display());
        for i in 0..is_last.len() {
            let flag = is_last.get(i).unwrap();
            if i != is_last.len() - 1 {
                if *flag {
                    print!("    ");
                } else {
                    print!("│   ");
                }
            } else {
                if *flag {
                    print!("└── ");
                } else {
                    print!("├── ");
                }
            }
        }
        if self.path.is_dir() {
            println!("\x1b[1;34m{}\x1b[0m", self.file_name);
        } else {
            println!("{}", self.file_name);
        }

        for child in &self.children {
            let now_flag = child.file_name == self.children.last().unwrap().file_name;
            let mut new_is_last = is_last.clone();
            new_is_last.push(now_flag);
            child.tree(new_is_last);
        }
    }

    pub fn organize(&mut self) {
        self.children.sort_by(|x, y| x.file_name.cmp(&y.file_name));
        for child in &mut self.children {
            child.organize();
        }
    }
}
