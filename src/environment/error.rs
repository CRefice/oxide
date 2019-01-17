use crate::token::Token;
use crate::value;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error<'a> {
    Value {
        err: value::Error<'a>,
        loc: (usize, usize),
    },
    VarNotFound(Token<'a>),
    WrongArgCount {
        expected: usize,
        found: usize,
        loc: (usize, usize),
    },
}

impl<'a> Error<'a> {
    pub fn location(&self) -> Option<(usize, usize)> {
        match self {
            Error::Value { loc, .. } => Some(*loc),
            Error::VarNotFound(token) => Some(token.loc),
            Error::WrongArgCount { loc, .. } => Some(*loc),
        }
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Value { err, loc, .. } => {
                let (row, col) = loc;
                write!(f, "{}:{}: {}", row, col, err)
            }
            Error::VarNotFound(name) => {
                let (row, col) = name.loc;
                write!(
                    f,
                    "{}:{}: Variable '{}' not found",
                    row,
                    col,
                    name.identifier()
                )
            }
            Error::WrongArgCount {
                expected,
                found,
                loc,
            } => {
                let (row, col) = loc;
                write!(
                    f,
                    "{}:{}: Wrong number of arguments supplied to function: found {}, expected {}",
                    row, col, found, expected
                )
            }
        }
    }
}
