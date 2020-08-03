use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::ops::Deref;

#[derive(Debug)]
pub struct CachePathError {
    err_type: CachePathErrorType,
}

impl CachePathError {
    pub fn new(err_type: CachePathErrorType) -> CachePathError {
        CachePathError { err_type: err_type }
    }
}

impl From<io::Error> for CachePathError {
    fn from(err: io::Error) -> Self {
        CachePathError::new(CachePathErrorType::Io(Box::new(err)))
    }
}

impl Display for CachePathError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let msg: Box<dyn Display> = match &self.err_type {
            CachePathErrorType::UnkownCachePath => Box::new("couldn't find cache path"),
            CachePathErrorType::Io(err) => Box::new(err),
        };
        write!(f, "{}", msg)
    }
}

impl Error for CachePathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.err_type {
            CachePathErrorType::UnkownCachePath => None,
            CachePathErrorType::Io(err) => Some(err.deref()),
        }
    }
}

#[derive(Debug)]
pub enum CachePathErrorType {
    UnkownCachePath,
    Io(Box<dyn Error>),
}
