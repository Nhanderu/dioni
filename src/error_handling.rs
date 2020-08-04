use std::{
    error,
    fmt::{self, Display, Formatter},
    io,
    ops::Deref,
    result,
};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnkownCachePath,
    AuthServerStopped,
    Io(io::Error),
    SpotifyError(Box<dyn error::Error>),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let msg: Box<dyn Display> = match &self {
            Error::AuthServerStopped => Box::new("server stopped without the auth code"),
            Error::UnkownCachePath => Box::new("couldn't find cache path"),
            Error::Io(err) => Box::new(err),
            Error::SpotifyError(err) => Box::new(err.deref()),
        };
        write!(f, "{}", msg)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self {
            Error::AuthServerStopped => None,
            Error::UnkownCachePath => None,
            Error::Io(err) => Some(err),
            Error::SpotifyError(err) => Some(err.deref()),
        }
    }
}
