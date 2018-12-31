use crate::token::Token;
use crate::value::Value;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Expression<'a> {
    Primary(Value),
    Grouping(Box<Expression<'a>>),
    Unary(Token<'a>, Box<Expression<'a>>),
    Binary(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
}

impl <'a> Display for Expression<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Primary(val) => write!(f, "{}", val),
            Expression::Grouping(expr) => write!(f, "({})", expr),
            Expression::Unary(op, expr) => write!(f, "{:?}{}", op, expr),
            Expression::Binary(left, op, right) => write!(f, "{} {:?} {}", left, op, right),
        }
    }
}
