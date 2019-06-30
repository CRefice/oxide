use std::ops::*;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Num(f64),
    Str(String),
    Bool(bool),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Num(x) => *x != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Bool(b) => *b,
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
