
use std::fmt;
use serde_json;

// Define a generic alias for a `Result` with the error type `ParseIntError`.
pub type Result<T> = std::result::Result<T, KvsError>;


#[derive(Debug, Clone)]
pub enum KvsError {
    KvPathNotFoundError,
    KeyNotFoundError,
    IoError,
    SerdeError,
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
            KvsError::SerdeError =>
                write!(f, "Serde json error"),
            }
    }
}

impl From<std::io::Error> for KvsError {
    fn from(err: std::io::Error) -> KvsError {
        KvsError::IoError
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::SerdeError
    }
}
