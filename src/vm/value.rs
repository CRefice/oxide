use std::cmp::Ordering;
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

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Num(_) => "Num",
            Value::Str(_) => "Str",
            Value::Bool(_) => "Bool",
            Value::Function { .. } => "Fn",
            Value::NativeFn { .. } => "NativeFn",
        }
    }

    pub fn cmp(&self, other: &Self) -> Result<Ordering> {
        self.partial_cmp(other).ok_or(Error::Comparison {
            a: self.clone(),
            b: other.clone(),
        })
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
            Value::Function { code_loc, arity } => write!(
                f,
                "Function {{ code_loc = {}, arity = {}, }}",
                code_loc, arity
            ),
            Value::NativeFn { .. } => write!(f, "NativeFn(..)"),
        }
    }
}

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

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        Value::Bool(!self.is_truthy())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (
                Value::Function {
                    code_loc: loc_a,
                    arity: arity_a,
                },
                Value::Function {
                    code_loc: loc_b,
                    arity: arity_b,
                },
            ) => (loc_a, arity_a) == (loc_b, arity_b),
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None,
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
    Comparison {
        a: Value,
        b: Value,
    },
    WrongCall(Value),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Unary { x, op } => write!(
                f,
                "Cannot apply operator '{}' to value of type '{}'",
                op,
                x.type_name()
            ),
            Error::Binary { a, b, op } => write!(
                f,
                "Cannot apply operator '{}' to values of type '{}' and '{}'",
                op,
                a.type_name(),
                b.type_name()
            ),
            Error::Comparison { a, b } => write!(
                f,
                "Cannot compare values of type '{}' and '{}'",
                a.type_name(),
                b.type_name()
            ),
            Error::WrongCall(val) => write!(
                f,
                "Cannot call value of type {} like a function",
                val.type_name()
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

type Result<T> = std::result::Result<T, Error>;
