use crate::environment;
use crate::parse;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Parse(parse::Error),
    Evaluate(environment::Error),
}

impl Error {
    pub fn location(&self) -> Option<(usize, usize)> {
        match self {
            Error::IO(_) => None,
            Error::Parse(err) => err.location(),
            Error::Evaluate(err) => err.location(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => err.fmt(f),
            Error::Parse(err) => err.fmt(f),
            Error::Evaluate(err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<parse::Error> for Error {
    fn from(e: parse::Error) -> Self {
        Error::Parse(e)
    }
}

impl From<environment::Error> for Error {
    fn from(e: environment::Error) -> Self {
        Error::Evaluate(e)
    }
}
