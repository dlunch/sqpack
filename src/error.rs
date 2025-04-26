use alloc::{fmt, format, string::String};
use core::result;

#[derive(Debug)]
pub enum SqPackReaderError {
    InvalidPath,
    NoSuchFolder,
    NoSuchFile,
    ReadError(String),
}

impl fmt::Display for SqPackReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SqPackReaderError::InvalidPath => f.write_str("Invalid path"),
            SqPackReaderError::NoSuchFolder => f.write_str("No such folder"),
            SqPackReaderError::NoSuchFile => f.write_str("No such file"),
            SqPackReaderError::ReadError(x) => f.write_str(&format!("Read error, {x}")),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for SqPackReaderError {
    fn from(x: std::io::Error) -> SqPackReaderError {
        SqPackReaderError::ReadError(x.to_string())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SqPackReaderError {}

pub type Result<T> = result::Result<T, SqPackReaderError>;
