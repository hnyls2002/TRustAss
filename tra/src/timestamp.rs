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

    pub fn leq_vec(&self, other: &VectorTime) -> bool {
        self.time <= *other.times.get(&self.id).unwrap_or(&0)
    }
}

impl From<HashMap<i32, i32>> for VectorTime {
    fn from(value: HashMap<i32, i32>) -> Self {
        Self { times: value }
    }
}

impl From<VectorTime> for HashMap<i32, i32> {
    fn from(value: VectorTime) -> Self {
        value.times
    }
}

impl VectorTime {
    pub fn from_singleton_time(create_time: &SingletonTime) -> Self {
        let mut times = HashMap::new();
        times.insert(create_time.id, create_time.time);
        Self { times }
    }

    // pub fn clear(&mut self) {
    //     self.times.clear();
    // }

    pub fn update_one(&mut self, id: i32, time: i32) {
        if let Some(old_time) = self.times.get_mut(&id) {
            assert!(time > *old_time);
            *old_time = time;
        } else {
            self.times.insert(id, time);
        }
    }

    pub fn check_max(&mut self, other: &Self) {
        for (id, time) in &other.times {
            if let Some(old) = self.times.get(id) {
                self.times.insert(*id, std::cmp::max(*old, *time));
            } else {
                self.times.insert(*id, *time);
            }
        }
    }

    pub fn leq(&self, other: &Self) -> bool {
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
