use crate::environment;
use crate::parse;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error<'a> {
    IO(std::io::Error),
    Parse(parse::Error<'a>),
    Evaluate(environment::Error<'a>),
}

impl<'a> Error<'a> {
    pub fn location(&self) -> Option<(usize, usize)> {
        match self {
            Error::IO(_) => None,
            Error::Parse(err) => err.location(),
            Error::Evaluate(err) => err.location(),
        }
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => err.fmt(f),
            Error::Parse(err) => err.fmt(f),
            Error::Evaluate(err) => err.fmt(f),
        }
    }
}

impl<'a> From<std::io::Error> for Error<'a> {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl<'a> From<parse::Error<'a>> for Error<'a> {
    fn from(e: parse::Error<'a>) -> Self {
        Error::Parse(e)
    }
}

impl<'a> From<environment::Error<'a>> for Error<'a> {
    fn from(e: environment::Error<'a>) -> Self {
        Error::Evaluate(e)
    }
}
