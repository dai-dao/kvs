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
        let mut current_offset = 0;
        match  load(&gen_file, &mut index) {
            Ok(offset) => current_offset = offset,
            Err(_) => ()
        }
        let file = fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&gen_file)?;
        let writer = MyWriter { buf : BufWriter::new(file), offset : current_offset };
        Ok (KvStore { index : index, writer : writer, reader : BufReader::new(File::open(&gen_file)?) })
    }

    // pass in reference of self to NOT move value
    pub fn remove(&mut self, key : String) -> Result<Option<String>> {
        // remove from in-memory index
        let value = self.index.remove(&key);
        // write to log
        let cmd = Command::Remove { key : key.to_owned() };
        to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        //
        match value {
            Some(command_offset) => {
                self.reader.seek(SeekFrom::Start(command_offset.start as u64))?;
                let entry_reader = (&mut self.reader).take(command_offset.end as u64);
                if let Command::Set { value, .. } = serde_json::from_reader(entry_reader)? {
                    Ok(Some(value))
                } else {
                    Err(KvsError::KeyNotFoundError)
                }
            },
            None => Err(KvsError::KeyNotFoundError)
        }
    }

    // pass in reference of self to NOT move value
    pub fn get(&mut self, key : String) -> Result<Option<String>> {
        if let Some(command_offset) = self.index.get(&key) {
            self.reader.seek(SeekFrom::Start(command_offset.start as u64))?;
            let entry_reader = (&mut self.reader).take(command_offset.end as u64);
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
        self.index.insert(key.to_owned(), CommandOffset { start : start, end : self.writer.offset-start });
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


fn load(path : &PathBuf, index : &mut HashMap<String, CommandOffset>) -> Result<usize> {
    let reader = BufReader::new(File::open(&path)?);
    let stream = &mut Deserializer::from_reader(reader).into_iter::<Command>();
    let mut current_offset = stream.byte_offset();
    while let Some(command) = stream.next() {
        let new_pos = stream.byte_offset();

        match command? {
            Command::Set { key , .. } => index.insert(key, CommandOffset { start : current_offset, 
                                                                              end : new_pos-current_offset }),
            Command::Remove { key } => index.remove(&key),
        };
        current_offset = new_pos;
    }

    Ok(current_offset)
}