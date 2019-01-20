use crate::token::Token;
use crate::value;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Value {
        err: value::Error,
        loc: (usize, usize),
    },
    VarNotFound(Token),
    WrongArgCount {
        expected: usize,
        found: usize,
        loc: (usize, usize),
    },
}

impl Error {
    pub fn location(&self) -> Option<(usize, usize)> {
        match self {
            Error::Value { loc, .. } => Some(*loc),
            Error::VarNotFound(token) => Some(token.loc),
            Error::WrongArgCount { loc, .. } => Some(*loc),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Value { err, .. } => write!(f, "{}", err),
            Error::VarNotFound(name) => write!(f, "Variable '{}' not found", name.identifier()),
            Error::WrongArgCount {
                expected, found, ..
            } => write!(
                f,
                "Wrong number of arguments supplied to function: found {}, expected {}",
                found, expected
            ),
        }
    }
}
