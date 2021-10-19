use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::fmt;
use serde_json::to_writer;
use std::io::BufReader;
use serde::{Serialize, Deserialize};
use std::ffi::OsStr;


#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key : String, value : String },
    Remove { key : String },
}


// Define a generic alias for a `Result` with the error type `ParseIntError`.
pub type Result<T> = std::result::Result<T, KvsError>;


#[derive(Debug, Clone)]
pub enum KvsError {
    KvPathNotFoundError,
    KeyNotFoundError,
    IoError,
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KvsError::KvPathNotFoundError =>
                write!(f, "Path not found"),
            KvsError::KeyNotFoundError =>
                write!(f, "Key not found to remove"),
            KvsError::IoError => 
                write!(f, "IO error"),
            }
    }
}

impl From<std::io::Error> for KvsError {
    fn from(err: std::io::Error) -> KvsError {
        KvsError::IoError
    }
}


#[derive(Default)]
pub struct KvStore {
    map : HashMap<String, String>,
    path : Option<PathBuf>,
}



fn sorted_gen_list(path : &Path) -> Result<Vec<u64>> {
    let mut gen_list : Vec<u64> = fs::read_dir(&path)?
                    .flat_map(|res| -> Result<_> { Ok(res?.path()) })
                    .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
                    .flat_map(|path| {
                        path.file_name()
                            .and_then(OsStr::to_str)
                            .map(|s| s.trim_end_matches(".log"))
                            .map(str::parse::<u64>)
                    })
                    .flatten()
                    .collect();
    gen_list.sort_unstable();
    Ok(gen_list)
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
        // write this to log on disk
        let cmd = Command::Set { key : key.to_owned(), value : value.to_owned() };
        //

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