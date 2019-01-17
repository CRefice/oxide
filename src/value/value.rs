use std::cmp::*;
use std::fmt::{self, Display, Formatter};
use std::ops::*;

use super::Error;
use super::Fn;

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Void,
    Num(f64),
    Bool(bool),
    Str(String),
    Fn(Fn<'a>),
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Num(x) => write!(f, "{}", x),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Fn(fun) => write!(
                f,
                "<{}>",
                match fun {
                    Fn::Native { .. } => "native function",
                    Fn::User { .. } => "function",
                }
            ),
        }
    }
}

type Result<'a, T> = std::result::Result<T, Error<'a>>;

impl<'a> Value<'a> {
    pub fn is_truthy(&self) -> Result<'a, bool> {
        match self {
            Value::Num(x) => Ok(*x != 0.0),
            Value::Str(s) => Ok(!s.is_empty()),
            Value::Bool(b) => Ok(*b),
            v => Err(Error::WrongType(Value::Bool(false), v.clone())),
        }
    }
}

impl<'a> Neg for Value<'a> {
    type Output = Result<'a, Value<'a>>;
    fn neg(self) -> Result<'a, Value<'a>> {
        match self {
            Value::Num(x) => Ok(Value::Num(-x)),
            x => Err(Error::UnaryOp(x, "-")),
        }
    }
}

impl<'a> Not for Value<'a> {
    type Output = Result<'a, Value<'a>>;
    fn not(self) -> Result<'a, Value<'a>> {
        match self {
            Value::Bool(x) => Ok(Value::Bool(!x)),
            x => Err(Error::UnaryOp(x, "!")),
        }
    }
}

impl<'a> Add for Value<'a> {
    type Output = Result<'a, Value<'a>>;
    fn add(self, other: Value<'a>) -> Result<'a, Value<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x + y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (Value::Str(x), Value::Num(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (a, b) => Err(Error::BinaryOp(a, b, "+")),
        }
    }
}

impl<'a> Sub for Value<'a> {
    type Output = Result<'a, Value<'a>>;
    fn sub(self, other: Value<'a>) -> Result<'a, Value<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x - y)),
            (a, b) => Err(Error::BinaryOp(a, b, "-")),
        }
    }
}

impl<'a> Mul for Value<'a> {
    type Output = Result<'a, Value<'a>>;
    fn mul(self, other: Value<'a>) -> Result<'a, Value<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x * y)),
            (a, b) => Err(Error::BinaryOp(a, b, "*")),
        }
    }
}

impl<'a> Div for Value<'a> {
    type Output = Result<'a, Value<'a>>;
    fn div(self, other: Value<'a>) -> Result<'a, Value<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x / y)),
            (a, b) => Err(Error::BinaryOp(a, b, "/")),
        }
    }
}

impl<'a> Value<'a> {
    pub fn equals(self, other: Value<'a>) -> Result<'a, Value<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Bool(x == y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x == y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x == y)),
            (a, b) => Err(Error::BinaryOp(a, b, "==")),
        }
    }

    pub fn compare(self, other: Value<'a>) -> Result<'a, Ordering> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(x.partial_cmp(&y).unwrap_or(Ordering::Less)),
            (a, b) => Err(Error::Comparison(a, b)),
        }
    }
}
