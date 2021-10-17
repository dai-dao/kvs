use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fmt;
use serde_json::to_writer;
use std::io::BufReader;


// Define a generic alias for a `Result` with the error type `ParseIntError`.
pub type Result<T> = std::result::Result<T, KvsError>;


#[derive(Debug, Clone)]
pub enum KvsError {
    KvPathNotFoundError,
    KeyNotFoundError
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KvsError::KvPathNotFoundError =>
                write!(f, "Path not found"),
            KvsError::KeyNotFoundError =>
                write!(f, "Key not found to remove"),
            }
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
            Err(KvsError::KvPathNotFoundError)
        }
    }

    pub fn new() -> KvStore {
        KvStore{ map : HashMap::new(), path : None }
    }

    // pass in reference of self to NOT move value
    pub fn remove(&mut self, key : String) -> Result<Option<String>> {
        let out = self.map.remove(&key);
        // write to file every remove
        match out {
            Some(val) => match self.path {
                    Some (ref path) => {
                        let tmp_file = File::create(path).expect("overwrite db.json");
                        to_writer(tmp_file, &self.map).ok();
                        Ok(Some(val))
                    } 
                    None => Ok(Some(val))
                }
            None => Err(KvsError::KeyNotFoundError)
        }
    }

    // pass in reference of self to NOT move value
    pub fn get(&mut self, key : String) -> Result<Option<String>> {
        match self.path {
            Some (ref path) => {
                if path.exists() {
                    let tmp_file = File::open(path).expect("read db.json");
                    let reader = BufReader::new(tmp_file);
                    self.map = serde_json::from_reader(reader).unwrap();
                }
            } 
            None => ()
        };
        // copy return value
        Ok(self.map.get(&key).cloned())
    }

    // pass in reference of self to NOT move value
    pub fn set(&mut self, key : String, value : String) -> Result<()> {
        self.map.insert(key, value);
        // write to file every insert
        match self.path {
            Some (ref path) => {
                let tmp_file = File::create(path).expect("overwrite db.json");
                to_writer(tmp_file, &self.map).ok();
            } 
            None => ()
        };
        Ok(())
    }
}