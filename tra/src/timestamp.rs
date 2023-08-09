use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct VectorTime {
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

    pub fn create_id(&self) -> i32 {
        self.id
    }

    pub fn time(&self) -> i32 {
        self.time
    }

    pub fn leq_vec(&self, other: &HashMap<i32, i32>) -> bool {
        self.time <= *other.get(&self.id).unwrap_or(&0)
    }
}

impl From<HashMap<i32, i32>> for VectorTime {
    fn from(value: HashMap<i32, i32>) -> Self {
        Self { times: value }
    }
}

impl VectorTime {
    pub fn from_singleton_time(create_time: &SingletonTime) -> Self {
        let mut times = HashMap::new();
        times.insert(create_time.id, create_time.time);
        Self { times }
    }

    pub fn extract_hashmap(&self) -> HashMap<i32, i32> {
        self.times.clone()
    }

    pub fn clear(&mut self) {
        self.times.clear();
    }

    pub fn update_one(&mut self, id: i32, time: i32) {
        if let Some(old_time) = self.times.get_mut(&id) {
            assert!(time > *old_time);
            *old_time = time;
        } else {
            self.times.insert(id, time);
        }
    }

    pub fn leq(&self, other: &HashMap<i32, i32>) -> bool {
        for (id, time) in &self.times {
            assert!(*time != 0);
            if let Some(other_time) = other.get(id) {
                if *time > *other_time {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn geq(&self, other: &HashMap<i32, i32>) -> bool {
        for (id, time) in other {
            assert!(*time != 0);
            if let Some(self_time) = self.times.get(id) {
                if *time > *self_time {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn geq_singleton(&self, id: i32, time: i32) -> bool {
        SingletonTime::new(id, time).leq_vec(&self.extract_hashmap())
    }

    pub fn display(&self) -> String {
        let mut ret = String::new();
        for (id, time) in &self.times {
            ret.push_str(&format!("({}, {}) ", id, time));
        }
        ret
    }
}
