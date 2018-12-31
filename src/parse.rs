use crate::expr::Expression;
use crate::token::{Lexer, Token};
use crate::value::Value;
use std::iter::Peekable;

pub struct Parser<'a> {
    iter: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Parser<'a> {
        Parser {
            iter: lex.peekable(),
        }
    }

    pub fn statement(&mut self) -> Expression<'a> {
        self.equality()
    }

    pub fn expression(&mut self) -> Expression<'a> {
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
            let expr = self.expression();
            if let Some(Token::RightParen) = self.iter.next() {
            } else {
                panic!("Expected closing parenthesis");
            }
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
