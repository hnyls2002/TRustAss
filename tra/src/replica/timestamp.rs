use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VectorTime {
    id: i32,
    times: HashMap<i32, i32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SingletonTime {
    id: i32,
    time: i32,
}

impl SingletonTime {
    pub fn new(id: i32, time: i32) -> Self {
        Self { id, time }
    }
}

impl VectorTime {
    pub fn new_empty(id: i32) -> Self {
        Self {
            id,
            times: HashMap::default(),
        }
    }

    pub fn from_singleton_time(create_time: &SingletonTime) -> Self {
        let mut times = HashMap::new();
        times.insert(create_time.id, create_time.time);
        Self {
            id: create_time.id,
            times,
        }
    }

    pub fn clear(&mut self) {
        self.times.clear();
    }

    pub fn update_singleton(&mut self, time: i32) {
        if let Some(old_time) = self.times.get_mut(&self.id) {
            assert!(time > *old_time);
            *old_time = time;
        } else {
            self.times.insert(self.id, time);
        }
    }

    // pub fn chkmax(&mut self, other: &Self) {
    //     for (id, time) in &other.times {
    //         if let Some(slef_time) = self.times.get(id) {
    //             self.times.insert(*id, (*slef_time).max(*time));
    //         } else {
    //             self.times.insert(*id, *time);
    //         }
    //     }
    // }

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

    pub fn display(&self) -> String {
        let mut ret = String::new();
        for (id, time) in &self.times {
            ret.push_str(&format!("({}, {}) ", id, time));
        }
        ret
    }
}
