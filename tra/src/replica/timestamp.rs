use std::collections::HashMap;

pub struct VectorTime {
    pub times: HashMap<u16, usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CreateTime {
    pub id: u16,
    pub time: usize,
}

impl CreateTime {
    pub fn new(id: u16, time: usize) -> Self {
        Self { id, time }
    }
}

impl Default for VectorTime {
    fn default() -> Self {
        Self {
            times: Default::default(),
        }
    }
}

impl VectorTime {
    pub fn from_create_time(create_time: &CreateTime) -> Self {
        let mut times = HashMap::new();
        times.insert(create_time.id, create_time.time);
        Self { times }
    }

    pub fn inside(&self, other: &VectorTime) -> bool {
        for (id, time) in &self.times {
            assert!(*time != 0);
            if let Some(other_time) = other.times.get(id) {
                if *time > *other_time {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
