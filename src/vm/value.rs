use std::fmt::{self, Debug, Display};
use std::ops::*;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Null,
    Num(f64),
    Str(String),
    Bool(bool),
    Function {
        code_loc: usize,
        arity: usize,
    },
    NativeFn {
        f: Rc<dyn Fn(&[Value]) -> Value>,
        arity: usize,
    },
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Num(x) => *x != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Num(x) => write!(f, "{}", x),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Function { .. } => write!(f, "fn"),
            Value::NativeFn { .. } => write!(f, "native fn"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Num(x) => write!(f, "Num({})", x),
            Value::Str(s) => write!(f, "Str({})", s),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Function { .. } => write!(f, "Function .."),
            Value::NativeFn { .. } => write!(f, "NativeFn(..)"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Unary {
        x: Value,
        op: &'static str,
    },
    Binary {
        a: Value,
        b: Value,
        op: &'static str,
    },
    WrongCall(Value),
}

type Result<T> = std::result::Result<T, Error>;

impl Add<Value> for Value {
    type Output = Result<Value>;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
            (a, b) => Err(Error::Binary { a, b, op: "+" }),
        }
    }
}

impl Sub<Value> for Value {
    type Output = Result<Value>;

    fn sub(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),
            (a, b) => Err(Error::Binary { a, b, op: "-" }),
        }
    }
}

impl Mul<Value> for Value {
    type Output = Result<Value>;

    fn mul(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
            (a, b) => Err(Error::Binary { a, b, op: "*" }),
        }
    }
}

impl Div<Value> for Value {
    type Output = Result<Value>;

    fn div(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a / b)),
            (a, b) => Err(Error::Binary { a, b, op: "/" }),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Num(x) => Ok(Value::Num(-x)),
            x => Err(Error::Unary { x, op: "-" }),
        }
    }
}
