
use std::fmt;
use serde_json;
use std::string::FromUtf8Error;

// Define a generic alias for a `Result` with the error type `ParseIntError`.
pub type Result<T> = std::result::Result<T, KvsError>;


#[derive(Debug, Clone)]
pub enum KvsError {
    KvPathNotFoundError,
    KeyNotFoundError,
    IoError,
    SerdeError,
    StringError(String),
    Sled(sled::Error),
    Utf8(FromUtf8Error)
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            KvsError::KvPathNotFoundError =>
                write!(f, "Path not found"),
            KvsError::KeyNotFoundError =>
                write!(f, "Key not found to remove"),
            KvsError::IoError => 
                write!(f, "IO error"),
            KvsError::SerdeError =>
                write!(f, "Serde json error"),
            KvsError::StringError(s) =>
                write!(f, "{}", s),
            KvsError::Sled(e) =>
                write!(f, "sled error: {}", e),
            KvsError::Utf8(e) => 
                write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl From<std::io::Error> for KvsError {
    fn from(_err: std::io::Error) -> KvsError {
        KvsError::IoError
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(err: FromUtf8Error) -> KvsError {
        KvsError::Utf8(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(_err: serde_json::Error) -> KvsError {
        KvsError::SerdeError
    }
}

impl From<sled::Error> for KvsError {
    fn from(err: sled::Error) -> KvsError {
        KvsError::Sled(err)
    }
}