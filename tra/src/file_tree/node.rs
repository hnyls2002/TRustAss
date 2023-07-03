pub enum NodeType {
    // no support for symlinks yet
    FILE,
    DIR,
}

pub struct Node {
    name: String,
    node_type: NodeType,
    children: Vec<Node>,
}

impl Node {
    pub fn new(name: String, node_type: NodeType) -> Node {
        Node {
            name,
            node_type,
            children: Vec::new(),
        }
    }
}
