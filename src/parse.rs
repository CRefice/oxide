use crate::expr::Expression;
use crate::stmt::Statement;
use crate::token::{self, Kind, Token};
use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;
use std::result;

#[derive(Debug)]
pub enum Error {
    Unexpected(Token),
    InvalidToken {
        expected: Kind,
        found: Token,
    },
    EndOfInput,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Unexpected(tok) => {
                write!(f, "Unexpected token: {}", tok)
            }
            Error::InvalidToken {
                expected, found, ..
            } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            Error::EndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}

impl Error {
    pub fn location(&self) -> Option<(usize, usize)> {
        match self {
            Error::Unexpected(tok) => Some(tok.loc),
            Error::InvalidToken { found, .. } => Some(found.loc),
            Error::EndOfInput => None,
        }
    }
}

type Result<T> = result::Result<T, Error>;

macro_rules! require {
    ($slf:expr, $pat:pat, $kind:expr, $ret:expr) => {{
        match $slf.iter.next() {
            Some(t @ Token { kind: $pat, .. }) => $ret(t),
            Some(t) => {
                return Err(Error::InvalidToken {
                    found: t,
                    expected: $kind,
                });
            }
            None => return Err(Error::EndOfInput),
        }
    }};
}

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    iter: Peekable<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(it: I) -> Self {
        Parser {
            iter: it.peekable(),
        }
    }

    pub fn program(&mut self) -> Result<Vec<Statement>> {
        let mut vec = Vec::new();
        while let Some(_) = self.iter.peek() {
            vec.push(self.declaration()?);
        }
        Ok(vec)
    }

    pub fn declaration(&mut self) -> Result<Statement> {
        match self.iter.peek().map(|t| &t.kind) {
            Some(token::Let) => self.var_declaration(),
            Some(token::Fn) => self.fn_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Statement> {
        self.iter.next(); // Let token
        require!(
            self,
            token::Identifier(_),
            token::Identifier(String::from("identifier")),
            |ident| require!(self, token::Equal, token::Equal, |_| Ok(
                Statement::VarDecl {
                    ident,
                    init: self.expression()?,
                }
            ))
        )
    }

    fn fn_declaration(&mut self) -> Result<Statement> {
        self.iter.next(); // Fn token
        require!(
            self,
            token::Identifier(_),
            token::Identifier(String::from("identifier")),
            |ident| require!(self, token::LeftParen, token::LeftParen, |_| {
                let params = if let Some(token::RightParen) = self.iter.peek().map(|t| &t.kind) {
                    self.iter.next();
                    Vec::new()
                } else {
                    self.params()?
                };
                Ok(Statement::FnDecl {
                    ident,
                    params,
                    body: Box::new(self.block()?),
                })
            })
        )
    }

    fn statement(&mut self) -> Result<Statement> {
        match self.iter.peek().map(|t| &t.kind) {
            Some(token::If) => self.if_statement(),
            Some(token::While) => self.while_statement(),
            Some(token::Return) => self.return_statement(),
            Some(token::LeftBrace) => self.block(),
            _ => self.expr_statement(),
        }
    }

    fn if_statement(&mut self) -> Result<Statement> {
        self.iter.next(); // If token
        let loc = self.iter.peek().map(|t| t.loc).unwrap_or((0, 0));
        let cond = self.expression()?;
        let succ = Box::new(self.block()?);
        let fail = if let Some(token::Else) = self.iter.peek().map(|t| &t.kind) {
            self.iter.next().unwrap();
            Some(Box::new(self.block()?))
        } else {
            None
        };
        Ok(Statement::If {
            cond,
            succ,
            fail,
            loc,
        })
    }

    fn while_statement(&mut self) -> Result<Statement> {
        self.iter.next(); // While token
        let loc = self.iter.peek().map(|t| t.loc).unwrap_or((0, 0));
        let cond = self.expression()?;
        let stmt = Box::new(self.block()?);
        Ok(Statement::While { cond, stmt, loc })
    }

    fn return_statement(&mut self) -> Result<Statement> {
        self.iter.next(); // Return token
        Ok(Statement::Return(
            if let Some(token::Semicolon) = self.iter.peek().map(|t| &t.kind) {
                self.iter.next();
                None
            } else {
                Some(self.expression()?)
            },
        ))
    }

    fn block(&mut self) -> Result<Statement> {
        require!(self, token::LeftBrace, token::LeftBrace, |_| ());
        let mut stmts = Vec::new();
        loop {
            if let Some(token) = self.iter.peek().map(|t| &t.kind) {
                match token {
                    token::RightBrace => {
                        self.iter.next();
                        break;
                    }
                    _ => {
                        stmts.push(self.declaration()?);
                    }
                }
            } else {
                return Err(Error::EndOfInput);
            }
        }
        Ok(Statement::Block(stmts))
    }

    fn params(&mut self) -> Result<Vec<Token>> {
        let mut vec = Vec::new();
        require!(
            self,
            token::Identifier(_),
            token::Identifier(String::from("identifier")),
            |t| vec.push(t)
        );
        loop {
            match self.iter.next() {
                Some(Token {
                    kind: token::RightParen,
                    ..
                }) => break,
                Some(Token {
                    kind: token::Comma, ..
                }) => {
                    require!(
                        self,
                        token::Identifier(_),
                        token::Identifier(String::from("identifier")),
                        |t| vec.push(t)
                    );
                }
                Some(found) => {
                    return Err(Error::InvalidToken {
                        expected: token::Comma,
                        found,
                    })
                }
                None => return Err(Error::EndOfInput),
            }
        }
        Ok(vec)
    }

    fn expr_statement(&mut self) -> Result<Statement> {
        Ok(Statement::Expression(self.expression()?))
    }

    pub fn expression(&mut self) -> Result<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression> {
        let expr = self.or()?;
        match self.iter.peek().map(|t| &t.kind) {
            Some(token::Equal) => {
                let found = self.iter.next().unwrap();
                let val = self.assignment()?;
                match expr {
                    Expression::Variable(ident) => Ok(Expression::Assignment {
                        ident,
                        val: Box::new(val),
                    }),
                    Expression::Indexing{operand, index, loc } => match *operand {
                        Expression::Variable(ident) => Ok(Expression::IndexingAssignment {
                            ident,
                            index,
                            val: Box::new(val),
                            loc
                        }),
                        _ => Err(Error::InvalidToken {
                            found,
                            expected: token::Identifier(String::from("identifier")),
                        }),
                    }
                    _ => Err(Error::InvalidToken {
                        found,
                        expected: token::Identifier(String::from("identifier")),
                    }),
                }
            }
            _ => Ok(expr),
        }
    }

    fn or(&mut self) -> Result<Expression> {
        let mut expr = self.and()?;
        while let Some(token::Or) = self.iter.peek().map(|t| &t.kind) {
            let op = self.iter.next().unwrap();
            let right = self.and()?;
            expr = Expression::Logical {
                a: Box::new(expr),
                b: Box::new(right),
                op,
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression> {
        let mut expr = self.equality()?;
        while let Some(token::And) = self.iter.peek().map(|t| &t.kind) {
            let op = self.iter.next().unwrap();
            let right = self.equality()?;
            expr = Expression::Logical {
                a: Box::new(expr),
                b: Box::new(right),
                op,
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression> {
        let mut expr = self.comparison()?;
        while let Some(token) = self.iter.peek().map(|t| &t.kind) {
            match token {
                token::EqualEqual | token::BangEqual => {
                    let op = self.iter.next().unwrap();
                    let right = self.comparison()?;
                    expr = Expression::Binary {
                        a: Box::new(expr),
                        b: Box::new(right),
                        op,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression> {
        let mut expr = self.addition()?;
        while let Some(token) = self.iter.peek().map(|t| &t.kind) {
            match token {
                token::Less | token::LessEqual | token::Greater | token::GreaterEqual => {
                    let op = self.iter.next().unwrap();
                    let right = self.addition()?;
                    expr = Expression::Binary {
                        a: Box::new(expr),
                        b: Box::new(right),
                        op,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expression> {
        let mut expr = self.multiplication()?;
        while let Some(token) = self.iter.peek().map(|t| &t.kind) {
            match token {
                token::Plus | token::Minus => {
                    let op = self.iter.next().unwrap();
                    let right = self.multiplication()?;
                    expr = Expression::Binary {
                        a: Box::new(expr),
                        b: Box::new(right),
                        op,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expression> {
        let mut expr = self.unary()?;
        while let Some(token) = self.iter.peek().map(|t| &t.kind) {
            match token {
                token::Star | token::Slash => {
                    let op = self.iter.next().unwrap();
                    let right = self.multiplication()?;
                    expr = Expression::Binary {
                        a: Box::new(expr),
                        b: Box::new(right),
                        op,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression> {
        match self.iter.peek().map(|t| &t.kind) {
            Some(token::Minus) | Some(token::Bang) => {
                let op = self.iter.next().unwrap();
                Ok(Expression::Unary {
                    op,
                    operand: Box::new(self.unary()?),
                })
            }
            _ => self.indexing(),
        }
    }

    fn indexing(&mut self) -> Result<Expression> {
        let mut expr = self.call()?;
        while let Some(token::LeftBracket) = self.iter.peek().map(|t| &t.kind) {
            let Token { loc, .. } = self.iter.next().unwrap();
            let index = self.expression()?;
            require!(self, token::RightBracket, token::RightBracket, |_| ());
            expr = Expression::Indexing {
                operand: Box::new(expr),
                index: Box::new(index),
                loc,
            };
        }
        Ok(expr)
    }

    fn call(&mut self) -> Result<Expression> {
        let mut expr = self.primary()?;
        while let Some(token::LeftParen) = self.iter.peek().map(|t| &t.kind) {
            let Token { loc, .. } = self.iter.next().unwrap();
            let args = if let Some(token::RightParen) = self.iter.peek().map(|t| &t.kind) {
                self.iter.next();
                Vec::new()
            } else {
                self.args()?
            };
            expr = Expression::Call {
                callee: Box::new(expr),
                args,
                loc,
            };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expression> {
        let token = self.iter.next();
        match token.as_ref().map(|t| &t.kind) {
            Some(token::LeftParen) => {
                let expr = self.expression()?;
                require!(self, token::RightParen, token::RightParen, |_| Ok(
                    Expression::Grouping(Box::new(expr))
                ))
            }
            Some(token::LeftBracket) => {
                Ok(Expression::Array(
                    if let Some(token::RightBracket) = self.iter.peek().map(|t| &t.kind) {
                        self.iter.next();
                        Vec::new()
                    } else {
                        self.array()?
                    },
                ))
            }
            Some(token::Literal(val)) => Ok(Expression::Literal(val.clone())),
            Some(token::Identifier(_)) => Ok(Expression::Variable(token.unwrap())),
            Some(_) => Err(Error::Unexpected(token.unwrap())),
            None => Err(Error::EndOfInput),
        }
    }

    fn array(&mut self) -> Result<Vec<Expression>> {
        let mut vec = vec![self.expression()?];
        loop {
            match self.iter.next() {
                Some(Token {
                    kind: token::RightBracket,
                    ..
                }) => break,
                Some(Token {
                    kind: token::Comma, ..
                }) => vec.push(self.expression()?),
                Some(found) => {
                    return Err(Error::InvalidToken {
                        expected: token::Comma,
                        found,
                    })
                }
                None => return Err(Error::EndOfInput),
            }
        }
        Ok(vec)
    }

    fn args(&mut self) -> Result<Vec<Expression>> {
        let mut vec = vec![self.expression()?];
        loop {
            match self.iter.next() {
                Some(Token {
                    kind: token::RightParen,
                    ..
                }) => break,
                Some(Token {
                    kind: token::Comma, ..
                }) => vec.push(self.expression()?),
                Some(found) => {
                    return Err(Error::InvalidToken {
                        expected: token::Comma,
                        found,
                    })
                }
                None => return Err(Error::EndOfInput),
            }
        }
        Ok(vec)
    }
}
