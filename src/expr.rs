use crate::scan::Token;
use crate::value::Value;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Expression<'a> {
    Literal(Value),
    Variable(&'a str),
    Assignment{name: &'a str, val: Box<Expression<'a>>},
    Grouping(Box<Expression<'a>>),
    Unary(Token<'a>, Box<Expression<'a>>),
    Binary(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
    Logical(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
}

impl <'a> Display for Expression<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Literal(val) => write!(f, "{}", val),
            Expression::Variable(var) => write!(f, "'{}'", var),
            Expression::Assignment{name, val} => write!(f, "{} = {}", name, val),
            Expression::Grouping(expr) => write!(f, "({})", expr),
            Expression::Unary(op, expr) => write!(f, "{:?}{}", op, expr),
            Expression::Binary(left, op, right) => write!(f, "{} {:?} {}", left, op, right),
            Expression::Logical(left, op, right) => write!(f, "{} {:?} {}", left, op, right),
        }
    }
}
