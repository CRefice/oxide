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
    InvalidExpression {
        expected: Expression<'a>,
        found: Expression<'a>,
    },
    EndOfInput,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidToken(t) => write!(f, "Unexpected token: {:?}", t),
            ParseError::MissingToken(t) => write!(f, "Expected token: {}", t),
            ParseError::InvalidExpression { expected, found } => {
                write!(f, "Expected expression: {:?}, found: {:?}", expected, found)
            }
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

    pub fn done(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    pub fn declaration(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        match self.iter.peek() {
            Some(Token::Let) => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        self.iter.next(); // Let token
        if let Some(Token::Identifier(name)) = self.iter.next() {
            match self.iter.next() {
                Some(Token::Equal) => Ok(Statement::VarDecl {
                    name,
                    init: self.expression()?,
                }),
                _ => Err(ParseError::MissingToken("=")),
            }
        } else {
            Err(ParseError::MissingToken("identifier"))
        }
    }

    fn statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        match self.iter.peek() {
            Some(Token::If) => self.if_statement(),
            Some(Token::LeftBrace) => self.block(),
            _ => self.expr_statement(),
        }
    }

    fn if_statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        self.iter.next(); // If token
        let cond = self.expression()?;
        let succ = Box::new(self.block()?);
        let fail = if let Some(Token::Else) = self.iter.peek() {
            self.iter.next().unwrap();
            Some(Box::new(self.block()?))
        } else {
            None
        };
        Ok(Statement::If { cond, succ, fail })
    }

    fn block(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        if let Some(Token::LeftBrace) = self.iter.next() {
        } else {
            return Err(ParseError::MissingToken("}"));
        }

        let mut stmts = Vec::new();
        while let Some(token) = self.iter.peek() {
            match token {
                Token::RightBrace => {
                    self.iter.next();
                    break;
                }
                _ => {
                    stmts.push(self.declaration()?);
                }
            }
        }
        Ok(Statement::Block(stmts))
    }

    fn expr_statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        Ok(Statement::Expression(self.expression()?))
    }

    pub fn expression(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let expr = self.or()?;
        match self.iter.peek() {
            Some(Token::Equal) => {
                self.iter.next();
                let val = self.assignment()?;
                match expr {
                    Expression::Variable(name) => Ok(Expression::Assignment {
                        name,
                        val: Box::new(val),
                    }),
                    expr => Err(ParseError::InvalidExpression {
                        found: expr,
                        expected: Expression::Assignment {
                            name: "variable",
                            val: Box::new(val),
                        },
                    }),
                }
            }
            _ => Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let mut expr = self.and()?;
        while let Some(Token::Or) = self.iter.peek() {
            let op = self.iter.next().unwrap();
            let right = self.and()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression<'a>, ParseError<'a>> {
        let mut expr = self.equality()?;
        while let Some(Token::And) = self.iter.peek() {
            let op = self.iter.next().unwrap();
            let right = self.equality()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
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
        } else if let Some(Token::Bool(b)) = tok {
            Ok(Expression::Literal(Value::Bool(b)))
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
