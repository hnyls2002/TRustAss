use async_recursion::async_recursion;

use super::{
    node::{Node, NodeStatus},
    Replica,
};

impl Replica {
    pub async fn tree(&self) {
        self.trees_collect.tree(Vec::new()).await;
    }
}

impl Node {
    #[async_recursion]
    pub async fn tree(&self, is_last: Vec<bool>) {
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
        if self.is_dir {
            print!("\x1b[1;34m{}\x1b[0m", self.file_name());
        } else {
            print!("{}", self.file_name());
        }

        print!(
            "  \x1b[33m{}\x1b[0m",
            self.data.read().await.mod_time.display()
        );
        println!(
            "  \x1b[32m{}\x1b[0m",
            self.data.read().await.sync_time.display()
        );

        let children = &self.data.read().await.children;
        let mut undeleted = Vec::new();

        for (_, child) in children {
            if child.data.read().await.status != NodeStatus::Deleted {
                undeleted.push(child);
            }
        }

        undeleted.sort_by(|a, b| (a.is_dir, a.file_name()).cmp(&(b.is_dir, b.file_name())));

        for child in &undeleted {
            let now_flag = child.file_name() == undeleted.last().unwrap().file_name();
            let mut new_is_last = is_last.clone();
            new_is_last.push(now_flag);
            child.tree(new_is_last).await;
        }
    }
}
