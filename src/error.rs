use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::ops::Deref;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, DioniError>;

#[derive(Debug)]
pub enum DioniError {
    UnkownCachePath,
    AuthServerStopped,
    Io(Box<dyn Error>),
}

impl From<io::Error> for DioniError {
    fn from(err: io::Error) -> Self {
        DioniError::Io(Box::new(err))
    }
}

impl Display for DioniError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let msg: Box<dyn Display> = match &self {
            DioniError::AuthServerStopped => Box::new("server stopped without the auth code"),
            DioniError::UnkownCachePath => Box::new("couldn't find cache path"),
            DioniError::Io(err) => Box::new(err),
        };
        write!(f, "{}", msg)
    }
}

impl Error for DioniError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            DioniError::AuthServerStopped => None,
            DioniError::UnkownCachePath => None,
            DioniError::Io(err) => Some(err.deref()),
        }
    }
}
