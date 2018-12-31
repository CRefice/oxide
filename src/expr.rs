use crate::token::{Lexer, Token};
use crate::value::{Value, ValueError};
use std::cmp::Ordering;
use std::iter::Peekable;

#[derive(Debug)]
pub enum Expression<'a> {
    Primary(Value),
    Grouping(Box<Expression<'a>>),
    Unary(Token<'a>, Box<Expression<'a>>),
    Binary(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
}

pub struct Parser<'a> {
    iter: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Parser<'a> {
        Parser {
            iter: lex.peekable(),
        }
    }

    pub fn parse(&mut self) -> Expression<'a> {
        self.equality()
    }

    fn equality(&mut self) -> Expression<'a> {
        let mut expr = self.comparison();
        while let Some(token) = self.iter.peek() {
            match token {
                Token::EqualEqual | Token::BangEqual => {
                    let token = self.iter.next().unwrap();
                    let right = self.addition();
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        expr
    }

    fn comparison(&mut self) -> Expression<'a> {
        let mut expr = self.addition();
        while let Some(token) = self.iter.peek() {
            match token {
                Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                    let token = self.iter.next().unwrap();
                    let right = self.addition();
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        expr
    }

    fn addition(&mut self) -> Expression<'a> {
        let mut expr = self.multiplication();
        while let Some(token) = self.iter.peek() {
            match token {
                Token::Plus | Token::Minus => {
                    let token = self.iter.next().unwrap();
                    let right = self.multiplication();
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        expr
    }

    fn multiplication(&mut self) -> Expression<'a> {
        let mut expr = self.unary();
        while let Some(token) = self.iter.peek() {
            match token {
                Token::Star | Token::Slash => {
                    let token = self.iter.next().unwrap();
                    let right = self.multiplication();
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        expr
    }

    fn unary(&mut self) -> Expression<'a> {
        match self.iter.peek() {
            Some(Token::Minus) | Some(Token::Bang) => {
                let token = self.iter.next().unwrap();
                Expression::Unary(token, Box::new(self.unary()))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Expression<'a> {
        let tok = self.iter.next();
        if let Some(Token::LeftParen) = tok {
            let expr = self.parse();
            self.iter.next();
            Expression::Grouping(Box::new(expr))
        } else if let Some(Token::Number(x)) = tok {
            Expression::Primary(Value::Num(x))
        } else if let Some(Token::StringLiteral(s)) = tok {
            Expression::Primary(Value::Str(s.to_string()))
        } else {
            panic!("Unrecognized expression");
        }
    }
}

pub fn evaluate(ex: Expression) -> Result<Value, ValueError> {
    match ex {
        Expression::Primary(x) => Ok(x),
        Expression::Grouping(b) => evaluate(*b),
        Expression::Unary(op, right) => {
            let val = evaluate(*right)?;
            match op {
                Token::Minus => -val,
                Token::Bang => !val,
                _ => panic!("Unrecognized unary operator")
            }
        }
        Expression::Binary(left, op, right) => {
            let left = evaluate(*left)?;
            let right = evaluate(*right)?;
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
                        _ => false
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
                        _ => false
                    };
                    Ok(Value::Bool(b))
                }
                _ => panic!("Unrecognized binary operator")
            }
        }
    }
}
