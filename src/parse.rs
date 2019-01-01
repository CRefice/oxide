use crate::expr::Expression;
use crate::stmt::Statement;
use crate::token::{Lexer, Token};
use crate::value::Value;
use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;

#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidToken(Token<'a>),
    MissingToken(&'static str),
    EndOfInput,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidToken(t) => write!(f, "Unexpected token: {:?}", t),
            ParseError::MissingToken(t) => write!(f, "Expected token: {}", t),
            ParseError::EndOfInput => write!(f, "Unexpected end of input"),
        }
    }
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

    pub fn declaration(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        match self.iter.peek() {
            Some(Token::Let) => {
                self.iter.next();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        if let Some(Token::Identifier(name)) = self.iter.next() {
            if let Some(Token::Equal) = self.iter.next() {
                let init = self.expression()?;
                if let Some(Token::Semicolon) = self.iter.next() {
                    Ok(Statement::VarDecl(name, init))
                } else {
                    Err(ParseError::MissingToken("="))
                }
            } else {
                Err(ParseError::MissingToken("="))
            }
        } else {
            Err(ParseError::MissingToken("identifier"))
        }
    }

    fn statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        self.expr_statement()
    }

    fn expr_statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        let expr = self.expression()?;
        if let Some(Token::Semicolon) = self.iter.peek() {
            self.iter.next();
            Ok(Statement::Expression(expr))
        } else {
            Err(ParseError::MissingToken(";"))
        }
    }

    pub fn expression(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let mut expr = self.comparison()?;
        while let Some(token) = self.iter.peek() {
            match token {
                Token::EqualEqual | Token::BangEqual => {
                    let token = self.iter.next().unwrap();
                    let right = self.comparison()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let mut expr = self.addition()?;
        while let Some(token) = self.iter.peek() {
            match token {
                Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                    let token = self.iter.next().unwrap();
                    let right = self.addition()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let mut expr = self.multiplication()?;
        while let Some(token) = self.iter.peek() {
            match token {
                Token::Plus | Token::Minus => {
                    let token = self.iter.next().unwrap();
                    let right = self.multiplication()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let mut expr = self.unary()?;
        while let Some(token) = self.iter.peek() {
            match token {
                Token::Star | Token::Slash => {
                    let token = self.iter.next().unwrap();
                    let right = self.multiplication()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        match self.iter.peek() {
            Some(Token::Minus) | Some(Token::Bang) => {
                let token = self.iter.next().unwrap();
                Ok(Expression::Unary(token, Box::new(self.unary()?)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let tok = self.iter.next();
        if let Some(Token::LeftParen) = tok {
            let expr = self.expression()?;
            if let Some(Token::RightParen) = self.iter.next() {
                Ok(Expression::Grouping(Box::new(expr)))
            } else {
                Err(ParseError::MissingToken(")"))
            }
        } else if let Some(Token::Number(x)) = tok {
            Ok(Expression::Literal(Value::Num(x)))
        } else if let Some(Token::StringLiteral(s)) = tok {
            Ok(Expression::Literal(Value::Str(s.to_string())))
        } else if let Some(Token::Identifier(var)) = tok {
            Ok(Expression::Variable(var))
        } else {
            match tok {
                Some(t) => Err(ParseError::InvalidToken(t)),
                None => Err(ParseError::EndOfInput),
            }
        }
    }
}
