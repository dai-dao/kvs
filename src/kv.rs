use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fmt;
use serde_json::to_writer;
use std::io::BufReader;


// Define a generic alias for a `Result` with the error type `ParseIntError`.
pub type Result<T> = std::result::Result<T, KvPathNotFoundError>;


#[derive(Debug, Clone)]
pub struct KvPathNotFoundError;
impl fmt::Display for KvPathNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Path not found")
    }
}



#[derive(Default)]
pub struct KvStore {
    map : HashMap<String, String>,
    path : Option<PathBuf>,
}


impl KvStore {
    pub fn open(path : &Path) -> Result<KvStore> {
        if path.exists() {
            let file_path = path.join("kv.json");
            Ok(KvStore { map : HashMap::new(), path : Some(file_path) })
        } else {
            Err(KvPathNotFoundError)
        }
    }

    pub fn new() -> KvStore {
        KvStore{ map : HashMap::new(), path : None }
    }

    // pass in reference of self to NOT move value
    pub fn remove(&mut self, key : String) {
        self.map.remove(&key);
        // write to file every insert, hmm not efficient
        match self.path {
            Some (ref path) => {
                let mut tmp_file = File::create(path).expect("overwrite db.json");
                to_writer(tmp_file, &self.map);
            } 
            None => ()
        };
    }

    // pass in reference of self to NOT move value
    pub fn get(&mut self, key : String) -> Option<String> {
        match self.path {
            Some (ref path) => {
                if path.exists() {
                    let mut tmp_file = File::open(path).expect("read db.json");
                    let reader = BufReader::new(tmp_file);
                    self.map = serde_json::from_reader(reader).unwrap();
                }
            } 
            None => ()
        };
        // copy return value
        self.map.get(&key).cloned()
    }

    // pass in reference of self to NOT move value
    pub fn set(&mut self, key : String, value : String) -> Result<()> {
        self.map.insert(key, value);
        // write to file every insert, hmm not efficient
        match self.path {
            Some (ref path) => {
                let mut tmp_file = File::create(path).expect("overwrite db.json");
                to_writer(tmp_file, &self.map);
            } 
            None => ()
        };
        Ok(())
    }
}