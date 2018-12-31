use self::ValueError::*;
use std::cmp::*;
use std::fmt::{self, Display, Formatter};
use std::ops::*;

#[derive(Debug)]
pub enum Value {
    Void,
    Num(f64),
    Bool(bool),
    Str(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Num(x) => write!(f, "{}", x),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug)]
pub enum ValueError {
    UnaryOp(Value, &'static str),
    BinaryOp(Value, Value, &'static str),
    Comparison(Value, Value),
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UnaryOp(val, op) => write!(f, "Cannot apply operator '{}' to the given operand ({:?})", op, val),
            BinaryOp(a, b, op) => write!(
                f,
                "Cannot apply operator '{}' to the given operands ('{:?}' and '{:?}')",
                op, a, b
            ),
            Comparison(a, b) => write!(
                f,
                "Cannot compare the given operands ('{:?}' and '{:?}')",
                a, b,
            ),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, ValueError>;
    fn neg(self) -> Result<Value, ValueError> {
        match self {
            Value::Num(x) => Ok(Value::Num(-x)),
            x => Err(UnaryOp(x, "-")),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, ValueError>;
    fn not(self) -> Result<Value, ValueError> {
        match self {
            Value::Bool(x) => Ok(Value::Bool(!x)),
            x => Err(UnaryOp(x, "!")),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, ValueError>;
    fn add(self, other: Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x + y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (a, b) => Err(BinaryOp(a, b, "+")),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, ValueError>;
    fn sub(self, other: Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x - y)),
            (a, b) => Err(BinaryOp(a, b, "-")),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, ValueError>;
    fn mul(self, other: Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x * y)),
            (a, b) => Err(BinaryOp(a, b, "*"))
        }
    }
}

impl Div for Value {
    type Output = Result<Value, ValueError>;
    fn div(self, other: Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Num(x / y)),
            (a, b) => Err(BinaryOp(a, b, "/"))
        }
    }
}

impl Value {
    pub fn equals(self, other: Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(Value::Bool(x == y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x == y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x == y)),
            (a, b) => Err(BinaryOp(a, b, "==")),
        }
    }

    pub fn compare(self, other: Value) -> Result<Ordering, ValueError> {
        match (self, other) {
            (Value::Num(x), Value::Num(y)) => Ok(x.partial_cmp(&y).unwrap_or(Ordering::Less)),
            (a, b) => Err(Comparison(a, b))
        }
    }
}
