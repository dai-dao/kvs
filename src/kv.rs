use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use serde_json::{to_writer, Deserializer};
use std::io::{self, Seek, Read, BufReader, BufWriter, Write, SeekFrom};
use serde::{Serialize, Deserialize};
use std::ffi::OsStr;
use crate::{KvsError, Result};
use std::str;



#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key : String, value : String },
    Remove { key : String },
}


pub struct KvStore {
    map : HashMap<String, String>,
    path : Option<PathBuf>,
    kv2 : KvStore2,
}

pub struct KvStore2 {
    index : HashMap<String, CommandOffset>,
    writer : MyWriter<File>,
    reader : BufReader<File>,
}

pub struct MyWriter<W : std::io::Write> {
    buf : BufWriter<W>,
    offset : usize, // keep track of where the writer is at now
}


#[derive(Debug)]
pub struct CommandOffset {
    start : usize,
    end : usize,
}

impl KvStore {
    pub fn open(path : &Path) -> Result<KvStore> {
        // v2
        fs::create_dir_all(path)?;
        // just load from 1 gen file for now
        let mut index : HashMap<String, CommandOffset> = HashMap::new();
        // load the gen file into index
        let gen_file = path.join("1.log");
        load(&gen_file, &mut index);
        let file = fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&gen_file)?;
        let writer = MyWriter { buf : BufWriter::new(file), offset : 0 };
        let kv2 = KvStore2 { index : index, writer : writer, reader : BufReader::new(File::open(&gen_file)?) };


        // 
        if path.exists() {
            let file_path = path.join("kv.json");
            Ok(KvStore { map : HashMap::new(), path : Some(file_path), kv2 : kv2 })
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
        if let Some(command_offset) = self.kv2.index.get(&key) {
            self.kv2.reader.seek(SeekFrom::Start(command_offset.start as u64))?;
            let mut entry_reader = (&mut self.kv2.reader).take(command_offset.end as u64);
            if let Command::Set { key, value } = serde_json::from_reader(entry_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::KeyNotFoundError)
            }
        } else {
            Ok(None)
        }

        // match self.path {
        //     Some (ref path) => {
        //         if path.exists() {
        //             let tmp_file = File::open(path).expect("read db.json");
        //             let reader = BufReader::new(tmp_file);
        //             self.map = serde_json::from_reader(reader).unwrap();
        //         }
        //     } 
        //     None => ()
        // };
        // Ok(self.map.get(&key).cloned())
    }

    // pass in reference of self to NOT move value
    pub fn set(&mut self, key : String, value : String) -> Result<()> {
        // write command to log on disk
        let cmd = Command::Set { key : key.to_owned(), value : value.to_owned() };
        let start = self.kv2.writer.offset;
        to_writer(&mut self.kv2.writer, &cmd);
        self.kv2.writer.flush()?;
        // add to index
        self.kv2.index.insert(key.to_owned(), CommandOffset { start : start, end : self.kv2.writer.offset-start });


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


impl<W: Write> Write for MyWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.buf.write(buf)?;
        self.offset += len;
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.buf.flush()
    }
}


fn load(path : &PathBuf, index : &mut HashMap<String, CommandOffset>) -> Result<()> {
    let reader = BufReader::new(File::open(&path)?);
    let stream = &mut Deserializer::from_reader(reader).into_iter::<Command>();
    let mut current_offset = stream.byte_offset();
    while let Some(command) = stream.next() {
        let new_pos = stream.byte_offset();
        match command? {
            Command::Set { key , value } => index.insert(key, CommandOffset { start : current_offset, 
                                                                              end : new_pos-current_offset }),
            Command::Remove { key } => index.remove(&key),
        };
        current_offset = new_pos;
    }

    Ok(())
}