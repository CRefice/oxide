use crate::expr::Expression;
use crate::token::Token;
use crate::value::{Value, ValueError};
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct Context<'a> {
    vars: HashMap<&'a str, Value>,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            vars: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, ex: Expression) -> Result<Value, ValueError> {
        match ex {
            Expression::Primary(x) => Ok(x),
            Expression::Grouping(b) => self.evaluate(*b),
            Expression::Unary(op, right) => {
                let val = self.evaluate(*right)?;
                match op {
                    Token::Minus => -val,
                    Token::Bang => !val,
                    _ => panic!("Unrecognized unary operator"),
                }
            }
            Expression::Binary(left, op, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;
                match op {
                    Token::Plus => left + right,
                    Token::Minus => left - right,
                    Token::Star => left * right,
                    Token::Slash => left / right,
                    Token::EqualEqual => left.equals(right),
                    Token::BangEqual => left.equals(right).and_then(|c| !c),
                    Token::Greater => {
                        let b = if let Ordering::Greater = left.compare(right)? {
                            true
                        } else {
                            false
                        };
                        Ok(Value::Bool(b))
                    }
                    Token::GreaterEqual => {
                        let b = match left.compare(right)? {
                            Ordering::Greater | Ordering::Equal => true,
                            _ => false,
                        };
                        Ok(Value::Bool(b))
                    }
                    Token::Less => {
                        let b = if let Ordering::Less = left.compare(right)? {
                            true
                        } else {
                            false
                        };
                        Ok(Value::Bool(b))
                    }
                    Token::LessEqual => {
                        let b = match left.compare(right)? {
                            Ordering::Less | Ordering::Equal => true,
                            _ => false,
                        };
                        Ok(Value::Bool(b))
                    }
                    _ => panic!("Unrecognized binary operator"),
                }
            }
        }
    }
}
