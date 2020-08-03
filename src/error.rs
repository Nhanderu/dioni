use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::ops::Deref;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, DioniError>;

#[derive(Debug)]
pub struct DioniError {
    err_type: DioniErrorType,
}

impl DioniError {
    pub fn new(err_type: DioniErrorType) -> DioniError {
        DioniError { err_type: err_type }
    }
}

impl From<io::Error> for DioniError {
    fn from(err: io::Error) -> Self {
        DioniError::new(DioniErrorType::Io(Box::new(err)))
    }
}

impl Display for DioniError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let msg: Box<dyn Display> = match &self.err_type {
            DioniErrorType::AuthServerStopped => Box::new("server stopped without the auth code"),
            DioniErrorType::UnkownCachePath => Box::new("couldn't find cache path"),
            DioniErrorType::Io(err) => Box::new(err),
        };
        write!(f, "{}", msg)
    }
}

impl Error for DioniError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.err_type {
            DioniErrorType::AuthServerStopped => None,
            DioniErrorType::UnkownCachePath => None,
            DioniErrorType::Io(err) => Some(err.deref()),
        }
    }
}

#[derive(Debug)]
pub enum DioniErrorType {
    UnkownCachePath,
    AuthServerStopped,
    Io(Box<dyn Error>),
}
