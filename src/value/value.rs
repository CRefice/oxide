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
    Array(Vec<Value<'a>>),
    Fn(Fn<'a>),
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Num(x) => write!(f, "{}", x),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(vals) => write!(
                f,
                "[{}]",
                vals.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
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
            Value::Array(a) => Ok(!a.is_empty()),
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
            (Value::Array(mut x), Value::Array(mut y)) => {
                x.append(&mut y);
                Ok(Value::Array(x))
            }
            (Value::Array(mut x), y) => {
                x.push(y);
                Ok(Value::Array(x))
            }
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

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Value<'a>) -> bool {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::Array(x), Value::Array(y)) => x == y,
            (a, b) => false,
        }
    }
}

impl<'a> Value<'a> {
    pub fn compare(&self, other: &Value<'a>) -> Result<'a, Ordering> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(x.partial_cmp(&y).unwrap_or(Ordering::Less)),
            (a, b) => Err(Error::Comparison(a.clone(), b.clone())),
        }
    }

    pub fn index(&self, index: &Value<'a>) -> Result<'a, Value<'a>> {
        match (self, index) {
            (Value::Array(x), Value::Num(i)) => {
                let idx = i.round() as isize;
                let idx = if idx < 0 {
                    (x.len() as isize) + idx
                } else {
                    idx
                } as usize;
                Ok(x[idx].clone())
            }
            (Value::Str(s), Value::Num(i)) => {
                let idx = i.round() as isize;
                let idx = if idx < 0 {
                    (s.len() as isize) + idx
                } else {
                    idx
                } as usize;
                Ok(Value::Str(s[idx..=idx].to_owned()))
            }
            (a, b) => Err(Error::Indexing(a.clone(), b.clone())),
        }
    }

    pub fn index_mut(&mut self, index: &Value<'a>) -> Result<'a, &mut Value<'a>> {
        match (self, index) {
            (Value::Array(x), Value::Num(i)) => {
                let idx = i.round() as isize;
                let idx = if idx < 0 {
                    (x.len() as isize) + idx
                } else {
                    idx
                } as usize;
                Ok(&mut x[idx])
            }
            (a, b) => Err(Error::IndexingMut(a.clone(), b.clone())),
        }
    }
}
