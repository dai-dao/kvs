use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use serde_json::{to_writer, Deserializer};
use std::io::{self, Seek, Read, BufReader, BufWriter, Write, SeekFrom};
use serde::{Serialize, Deserialize};
use std::ffi::OsStr;
use crate::{KvsError, Result};
use std::str;
use std::sync::{Arc, Mutex};

use super::KvsEngine;


const COMPACTION_THRESHOLD: usize = 1024 * 1024;


#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key : String, value : String },
    Remove { key : String },
}

// need arc mutex because it'll be owned by many ownerships and shared between multiple threads
// so need thread-safe and synchronization lock for write protection 
pub struct KvStore {
    path : PathBuf,
    writer: Arc<Mutex<StoreWriter>> 
}

pub struct StoreWriter {
    index : HashMap<String, CommandOffset>,
    writer : MyWriter<File>,
    uncompacted : usize,
    current_gen : u64,
    path : PathBuf,
    readers : HashMap<u64, BufReader<File>>,
}

pub struct MyWriter<W : std::io::Write> {
    buf : BufWriter<W>,
    offset : usize, // keep track of where the writer is at now
}

#[derive(Debug)]
pub struct CommandOffset {
    gen : u64,
    start : usize,
    end : usize,
}

impl KvStore {
    pub fn open(path : &Path) -> Result<KvStore> {
        fs::create_dir_all(path)?;
        let mut index : HashMap<String, CommandOffset> = HashMap::new();
        let mut readers : HashMap<u64, BufReader<File>> = HashMap::new();
        let gen_files = get_sorted_gen_files(path)?;
        let mut uncompacted = 0;
        for &gen in &gen_files {
            let mut reader = BufReader::new(File::open(log_path(path, gen))?);
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }
        // use latest gen file to write
        let last_gen = gen_files.last().unwrap_or(&0) + 1;
        let file = fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&log_path(path, last_gen))?;
        let writer = MyWriter { buf : BufWriter::new(file), offset : 0 };
        readers.insert(last_gen, BufReader::new(File::open(log_path(path, last_gen))?));
        let store_writer = StoreWriter { index : index, writer : writer, readers : readers, 
                                        uncompacted : uncompacted, current_gen : last_gen, path : path.into() };
        Ok (KvStore { path : path.into(), writer: Arc::new(Mutex::new(store_writer)) })
    }
}

impl StoreWriter {
    // pass in reference of self to NOT move value
    pub fn remove(&mut self, key : String) -> Result<()> {
        // remove from in-memory index
        let value = self.index.remove(&key);
        match value {
            Some(command_offset) => {
                // write to log
                let cmd = Command::Remove { key : key.to_owned() };
                to_writer(&mut self.writer, &cmd)?;
                self.writer.flush()?;
                self.uncompacted += command_offset.end;
                Ok(())            
            },
            None => Err(KvsError::KeyNotFoundError)
        }
    }

    // pass in reference of self to NOT move value
    pub fn get(&mut self, key : String) -> Result<Option<String>> {
        if let Some(command_offset) = self.index.get(&key) {
            let reader = self.readers.get_mut(&command_offset.gen).expect("Can not find log reader");
            reader.seek(SeekFrom::Start(command_offset.start as u64))?;
            let entry_reader = reader.take(command_offset.end as u64);
            if let Command::Set { value, .. } = serde_json::from_reader(entry_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::KeyNotFoundError)
            }
        } else {
            Ok(None)
        }
    }

    // pass in reference of self to NOT move value
    pub fn set(&mut self, key : String, value : String) -> Result<()> {
        // write command to log on disk
        let cmd = Command::Set { key : key.to_owned(), value : value.to_owned() };
        let start = self.writer.offset;
        to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Some(old_cmd) = self.index.insert(key.to_owned(), 
                                    CommandOffset { gen : self.current_gen, start : start, end : self.writer.offset-start }) {
            self.uncompacted += old_cmd.end;

            if self.uncompacted > COMPACTION_THRESHOLD {
                self.compact()?;
            }
        }
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compaction file.
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = self.new_gen_file(self.current_gen)?;
        //
        let mut compaction_writer = self.new_gen_file(compaction_gen)?;

        let mut new_pos = 0; // pos in the new log file.
        // only take what's in the index, any stale entries will be ignored
        for command_offset in &mut self.index.values_mut() {
            let reader = self
                .readers
                .get_mut(&command_offset.gen)
                .expect("Cannot find log reader");
            reader.seek(SeekFrom::Start(command_offset.start as u64))?;
            let mut entry_reader = reader.take(command_offset.end as u64);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *command_offset = CommandOffset { gen : compaction_gen, start : new_pos, end : len as usize };
            new_pos += len as usize;
        }
        compaction_writer.flush()?;

        // remove stale entries
        let stale_gens : Vec<u64> = self.readers
                                            .keys()
                                            .filter(|&&gen| gen < compaction_gen)
                                            .cloned()
                                            .collect();
        for gen in stale_gens {
            self.readers.remove(&gen);
            fs::remove_file(log_path(self.path.as_path(), gen))?;
        }

        self.uncompacted = 0;

        Ok(())
    }

    // add new reader to current gen, and return writer
    fn new_gen_file(&mut self, gen : u64) -> Result<MyWriter<File>> {
        let file = fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&log_path(self.path.as_path(), gen))?;
        self.readers.insert(gen, BufReader::new(File::open(log_path(self.path.as_path(), gen))?));
        return Ok(MyWriter { buf : BufWriter::new(file), offset : 0 })
    }

}


impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key,value)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        self.writer.lock().unwrap().get(key)
    }

    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }
}


impl Clone for KvStore {
    fn clone(&self) -> Self {
        KvStore {
            path : self.path.clone(),
            writer: Arc::clone(&self.writer)
        }
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


fn log_path(path : &Path, gen : u64) -> PathBuf {
    return path.join(gen.to_string()+".log")
}


fn get_sorted_gen_files(path : &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = fs::read_dir(&path)?
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


fn load(gen : u64, reader : &mut BufReader<File>, index : &mut HashMap<String, CommandOffset>) -> Result<usize> {
    let stream = &mut Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;
    let mut current_offset = stream.byte_offset();
    while let Some(command) = stream.next() {
        let new_pos = stream.byte_offset();

        match command? {
            Command::Set { key , .. } => {
                if let Some(old_cmd) = index.insert(key, CommandOffset { gen : gen, start : current_offset, end : new_pos-current_offset }) {
                    uncompacted += old_cmd.end;
                }
            },
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.end;
                }
                uncompacted += new_pos - current_offset;
            }
        };
        current_offset = new_pos;
    }

    Ok(uncompacted)
}