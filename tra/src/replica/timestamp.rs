use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VectorTime {
    times: HashMap<u16, usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SingletonTime {
    id: u16,
    time: usize,
}

impl SingletonTime {
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
    pub fn from_singleton_time(create_time: &SingletonTime) -> Self {
        let mut times = HashMap::new();
        times.insert(create_time.id, create_time.time);
        Self { times }
    }

    pub fn update(&mut self, id: u16, time: usize) {
        self.times.insert(id, time);
    }

    pub fn clear(&mut self) {
        self.times.clear();
    }

    pub fn chkmax(&mut self, other: &Self) {
        for (id, time) in &other.times {
            if let Some(slef_time) = self.times.get(id) {
                self.times.insert(*id, (*slef_time).max(*time));
            } else {
                self.times.insert(*id, *time);
            }
        }
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
