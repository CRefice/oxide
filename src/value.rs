use self::ValueError::*;
use crate::scan::Token;
use crate::stmt::Statement;
use std::cmp::*;
use std::fmt::{self, Display, Formatter};
use std::ops::{self, *};

#[derive(Clone)]
pub enum Fn<'a> {
    Native {
        arity: usize,
        f: &'a dyn ops::Fn(Vec<Value<'a>>) -> Value<'a>,
    },
    User {
        closure: usize,
        params: Vec<Token<'a>>,
        body: Box<Statement<'a>>,
    },
}

#[macro_export]
macro_rules! function {
    ( $($x:ident),* , $body:expr ) => {
            |vec: Vec<value::Value>| {
                let mut _i = vec.into_iter();
                $(
                    let $x = _i.next().unwrap();
                )*
                $body
            };
    };
}

impl<'a> fmt::Debug for Fn<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Fn::Native { .. } => write!(f, "<NativeFn>"),
            Fn::User { .. } => write!(f, "<UserFn>"),
        }
    }
}

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

#[derive(Debug)]
pub enum ValueError<'a> {
    UnaryOp(Value<'a>, &'static str),
    BinaryOp(Value<'a>, Value<'a>, &'static str),
    Comparison(Value<'a>, Value<'a>),
    WrongType(Value<'a>, Value<'a>),
}

impl<'a> Display for ValueError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UnaryOp(val, op) => write!(
                f,
                "Cannot apply operator '{}' to the given operand (val)",
                op,
            ),
            BinaryOp(a, b, op) => write!(
                f,
                "Cannot apply operator '{}' to the given operands ('a' and 'b')",
                op,
            ),
            Comparison(a, b) => write!(f, "Cannot compare the given operands ('a' and 'b')",),
            WrongType(a, b) => write!(f, "Type mismatch: expected a, got b"),
        }
    }
}

impl<'a> Value<'a> {
    pub fn as_bool(self) -> Result<bool, ValueError<'a>> {
        match self {
            Value::Bool(b) => Ok(b),
            v => Err(WrongType(Value::Bool(false), v)),
        }
    }
}

impl<'a> Neg for Value<'a> {
    type Output = Result<Value<'a>, ValueError<'a>>;
    fn neg(self) -> Result<Value<'a>, ValueError<'a>> {
        match self {
            Value::Num(x) => Ok(Value::Num(-x)),
            x => Err(UnaryOp(x, "-")),
        }
    }
}

impl<'a> Not for Value<'a> {
    type Output = Result<Value<'a>, ValueError<'a>>;
    fn not(self) -> Result<Value<'a>, ValueError<'a>> {
        match self {
            Value::Bool(x) => Ok(Value::Bool(!x)),
            x => Err(UnaryOp(x, "!")),
        }
    }
}

impl<'a> Add for Value<'a> {
    type Output = Result<Value<'a>, ValueError<'a>>;
    fn add(self, other: Value<'a>) -> Result<Value<'a>, ValueError<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x + y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (a, b) => Err(BinaryOp(a, b, "+")),
        }
    }
}

impl<'a> Sub for Value<'a> {
    type Output = Result<Value<'a>, ValueError<'a>>;
    fn sub(self, other: Value<'a>) -> Result<Value<'a>, ValueError<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x - y)),
            (a, b) => Err(BinaryOp(a, b, "-")),
        }
    }
}

impl<'a> Mul for Value<'a> {
    type Output = Result<Value<'a>, ValueError<'a>>;
    fn mul(self, other: Value<'a>) -> Result<Value<'a>, ValueError<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x * y)),
            (a, b) => Err(BinaryOp(a, b, "*")),
        }
    }
}

impl<'a> Div for Value<'a> {
    type Output = Result<Value<'a>, ValueError<'a>>;
    fn div(self, other: Value<'a>) -> Result<Value<'a>, ValueError<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x / y)),
            (a, b) => Err(BinaryOp(a, b, "/")),
        }
    }
}

impl<'a> Value<'a> {
    pub fn equals(self, other: Value<'a>) -> Result<Value<'a>, ValueError<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Bool(x == y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x == y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x == y)),
            (a, b) => Err(BinaryOp(a, b, "==")),
        }
    }

    pub fn compare(self, other: Value<'a>) -> Result<Ordering, ValueError<'a>> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(x.partial_cmp(&y).unwrap_or(Ordering::Less)),
            (a, b) => Err(Comparison(a, b)),
        }
    }
}
