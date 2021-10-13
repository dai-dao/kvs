use std::collections::HashMap;


#[derive(Default)]
pub struct KvStore {
    map : HashMap<String, String>,
}


impl KvStore {
    pub fn new() -> KvStore {
        KvStore{ map : HashMap::new() }
    }

    // pass in reference of self to NOT move value
    pub fn remove(&mut self, key : String) {
        self.map.remove(&key);
    }

    // pass in reference of self to NOT move value
    pub fn get(&self, key : String) -> Option<String> {
        // copy return value
        self.map.get(&key).cloned()
    }

    // pass in reference of self to NOT move value
    pub fn set(&mut self, key : String, value : String) {
        self.map.insert(key, value);
    }
}