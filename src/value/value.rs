use std::cmp::*;
use std::fmt::{self, Display, Formatter};
use std::ops::*;

use super::Error;
use super::Fn;

#[derive(Clone, Debug)]
pub enum Value {
    Void,
    Num(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Fn(Fn),
}

impl Display for Value {
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

pub type Result<T> = std::result::Result<T, Error>;

impl Value {
    pub fn is_truthy(&self) -> Result<bool> {
        match self {
            Value::Num(x) => Ok(*x != 0.0),
            Value::Str(s) => Ok(!s.is_empty()),
            Value::Bool(b) => Ok(*b),
            Value::Array(a) => Ok(!a.is_empty()),
            v => Err(Error::WrongType(Value::Bool(false), v.clone())),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value>;
    fn neg(self) -> Result<Value> {
        match self {
            Value::Num(x) => Ok(Value::Num(-x)),
            x => Err(Error::UnaryOp(x, "-")),
        }
    }
}

impl Not for Value {
    type Output = Result<Value>;
    fn not(self) -> Result<Value> {
        match self {
            Value::Bool(x) => Ok(Value::Bool(!x)),
            x => Err(Error::UnaryOp(x, "!")),
        }
    }
}

impl Add for Value {
    type Output = Result<Value>;
    fn add(self, other: Value) -> Result<Value> {
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

impl Sub for Value {
    type Output = Result<Value>;
    fn sub(self, other: Value) -> Result<Value> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x - y)),
            (a, b) => Err(Error::BinaryOp(a, b, "-")),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value>;
    fn mul(self, other: Value) -> Result<Value> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x * y)),
            (a, b) => Err(Error::BinaryOp(a, b, "*")),
        }
    }
}

impl Div for Value {
    type Output = Result<Value>;
    fn div(self, other: Value) -> Result<Value> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x / y)),
            (a, b) => Err(Error::BinaryOp(a, b, "/")),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::Array(x), Value::Array(y)) => x == y,
            _ => false,
        }
    }
}

impl Value {
    pub fn compare(&self, other: &Value) -> Result<Ordering> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(x.partial_cmp(&y).unwrap_or(Ordering::Less)),
            (a, b) => Err(Error::Comparison(a.clone(), b.clone())),
        }
    }

    pub fn index(&self, index: &Value) -> Result<Value> {
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

    pub fn index_mut(&mut self, index: &Value) -> Result<&mut Value> {
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
