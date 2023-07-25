use std::collections::HashMap;

pub struct SyncTime {
    pub time: HashMap<u16, usize>,
}

pub struct ModTime {
    pub time: HashMap<u16, usize>,
}

pub struct CreateTime {
    pub time: usize,
}
