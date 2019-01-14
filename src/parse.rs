use crate::expr::Expression;
use crate::scan::Token;
use crate::stmt::Statement;
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;
use std::result;

#[derive(Debug)]
pub enum ParseError<'a> {
    Unexpected {
        tok: Token<'a>,
        loc: usize,
    },
    InvalidToken {
        expected: Token<'a>,
        found: Token<'a>,
        loc: usize,
    },
    EndOfInput,
}

impl<'a> ParseError<'a> {
    pub fn location(&self) -> usize {
        match self {
            ParseError::Unexpected { loc, .. } => *loc,
            ParseError::InvalidToken { loc, .. } => *loc,
            ParseError::EndOfInput => usize::max_value(),
        }
    }
}

fn to_str(t: &Token) -> &'static str {
    match t {
        Token::Literal(_) => "literal",
        Token::Identifier(_) => "identifier",
        Token::Let => "'let'",
        Token::Fn => "'fn'",
        Token::If => "'if'",
        Token::Else => "'else'",
        Token::While => "'while'",
        Token::And => "'and'",
        Token::Or => "'or'",
        Token::Return => "'return'",
        Token::Comma => "','",
        Token::Semicolon => "';'",
        Token::Plus => "'+'",
        Token::Minus => "'-'",
        Token::Star => "'*'",
        Token::Slash => "'/'",
        Token::Bang => "'!'",
        Token::Equal => "'='",
        Token::EqualEqual => "'=='",
        Token::BangEqual => "'!='",
        Token::Greater => "'>'",
        Token::GreaterEqual => "'>='",
        Token::Less => "'<'",
        Token::LessEqual => "'<='",
        Token::LeftParen => "'('",
        Token::RightParen => "')'",
        Token::LeftBracket => "'['",
        Token::RightBracket => "']'",
        Token::LeftBrace => "'{'",
        Token::RightBrace => "'}'",
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::Unexpected { tok, .. } => write!(f, "Unexpected token: {}", to_str(tok)),
            ParseError::InvalidToken {
                expected, found, ..
            } => write!(f, "Expected {}, found {}", to_str(expected), to_str(found)),
            ParseError::EndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}

type Result<'a, T> = result::Result<T, ParseError<'a>>;

macro_rules! match_token {
    ($val:expr, $exp:pat, $expr:expr, $ret:expr) => {
        match $val {
            Some((_, $exp)) => $ret,
            Some((loc, x)) => {
                return Err(ParseError::InvalidToken {
                    found: x,
                    expected: $expr,
                    loc,
                })
            }
            None => return Err(ParseError::EndOfInput),
        }
    };
}

pub struct Parser<'a, I>
where
    I: Iterator<Item = (usize, Token<'a>)>,
{
    iter: Peekable<I>,
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = (usize, Token<'a>)>,
{
    pub fn new(it: I) -> Self {
        Parser {
            iter: it.peekable(),
        }
    }

    pub fn program(&mut self) -> Result<'a, Vec<Statement<'a>>> {
        let mut vec = Vec::new();
        while let Some(_) = self.iter.peek() {
            vec.push(self.declaration()?);
        }
        Ok(vec)
    }

    pub fn declaration(&mut self) -> Result<'a, Statement<'a>> {
        match self.iter.peek() {
            Some((_, Token::Let)) => self.var_declaration(),
            Some((_, Token::Fn)) => self.fn_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<'a, Statement<'a>> {
        self.iter.next(); // Let token
        match_token!(
            self.iter.next(),
            ident@Token::Identifier(_),
            Token::Identifier(Cow::from("identifier")),
            {
                match_token!(self.iter.next(), Token::Equal, Token::Equal, {
                    Ok(Statement::VarDecl {
                        ident,
                        init: self.expression()?,
                    })
                })
            }
        )
    }

    fn fn_declaration(&mut self) -> Result<'a, Statement<'a>> {
        self.iter.next(); // Fn token
        match_token!(
            self.iter.next(),
            ident@Token::Identifier(_),
            Token::Identifier(Cow::from("identifier")),
            {
                match_token!(self.iter.next(), Token::LeftParen, Token::LeftParen, {
                    let params = if let Some((_, Token::RightParen)) = self.iter.peek() {
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
            }
        )
    }

    fn statement(&mut self) -> Result<'a, Statement<'a>> {
        match self.iter.peek() {
            Some((_, Token::If)) => self.if_statement(),
            Some((_, Token::While)) => self.while_statement(),
            Some((_, Token::Return)) => self.return_statement(),
            Some((_, Token::LeftBrace)) => self.block(),
            _ => self.expr_statement(),
        }
    }

    fn if_statement(&mut self) -> Result<'a, Statement<'a>> {
        self.iter.next(); // If token
        let cond = self.expression()?;
        let succ = Box::new(self.block()?);
        let fail = if let Some((_, Token::Else)) = self.iter.peek() {
            self.iter.next().unwrap();
            Some(Box::new(self.block()?))
        } else {
            None
        };
        Ok(Statement::If { cond, succ, fail })
    }

    fn while_statement(&mut self) -> Result<'a, Statement<'a>> {
        self.iter.next(); // While token
        let cond = self.expression()?;
        let stmt = Box::new(self.block()?);
        Ok(Statement::While { cond, stmt })
    }

    fn return_statement(&mut self) -> Result<'a, Statement<'a>> {
        self.iter.next(); // Return token
        Ok(Statement::Return(if let Some((_, Token::Semicolon)) = self.iter.peek() {
            self.iter.next();
            None
        } else {
            Some(self.expression()?)
        }))
    }

    fn block(&mut self) -> Result<'a, Statement<'a>> {
        match_token!(self.iter.next(), Token::LeftBrace, Token::LeftBrace, ());
        let mut stmts = Vec::new();
        loop {
            if let Some((_, token)) = self.iter.peek() {
                match token {
                    Token::RightBrace => {
                        self.iter.next();
                        break;
                    }
                    _ => {
                        stmts.push(self.declaration()?);
                    }
                }
            } else {
                return Err(ParseError::EndOfInput);
            }
        }
        Ok(Statement::Block(stmts))
    }

    fn params(&mut self) -> Result<'a, Vec<Token<'a>>> {
        let mut vec = Vec::new();
        match_token!(
            self.iter.next(),
            ident@Token::Identifier(_),
            Token::Identifier(Cow::from("identifier")),
            {
                vec.push(ident);
            }
        );
        loop {
            match self.iter.next() {
                Some((_, Token::RightParen)) => break,
                Some((_, Token::Comma)) => {
                    match_token!(
                        self.iter.next(),
                        ident@Token::Identifier(_),
                        Token::Identifier(Cow::from("identifier")),
                        {
                            vec.push(ident);
                        }
                    );
                }
                Some((loc, found)) => {
                    return Err(ParseError::InvalidToken {
                        expected: Token::Comma,
                        found,
                        loc,
                    })
                }
                None => return Err(ParseError::EndOfInput),
            }
        }
        Ok(vec)
    }

    fn expr_statement(&mut self) -> Result<'a, Statement<'a>> {
        Ok(Statement::Expression(self.expression()?))
    }

    pub fn expression(&mut self) -> Result<'a, Expression<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<'a, Expression<'a>> {
        let expr = self.or()?;
        match self.iter.peek() {
            Some((_, Token::Equal)) => {
                let (loc, token) = self.iter.next().unwrap();
                let val = self.assignment()?;
                match expr {
                    Expression::Variable(ident) => Ok(Expression::Assignment {
                        ident,
                        val: Box::new(val),
                    }),
                    _ => Err(ParseError::InvalidToken {
                        found: token,
                        expected: Token::Identifier(Cow::from("var")),
                        loc,
                    }),
                }
            }
            _ => Ok(expr),
        }
    }

    fn or(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.and()?;
        while let Some((_, Token::Or)) = self.iter.peek() {
            let (_, op) = self.iter.next().unwrap();
            let right = self.and()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.equality()?;
        while let Some((_, Token::And)) = self.iter.peek() {
            let (_, op) = self.iter.next().unwrap();
            let right = self.equality()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.comparison()?;
        while let Some((_, token)) = self.iter.peek() {
            match token {
                Token::EqualEqual | Token::BangEqual => {
                    let (_, token) = self.iter.next().unwrap();
                    let right = self.comparison()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.addition()?;
        while let Some((_, token)) = self.iter.peek() {
            match token {
                Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                    let (_, token) = self.iter.next().unwrap();
                    let right = self.addition()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn addition(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.multiplication()?;
        while let Some((_, token)) = self.iter.peek() {
            match token {
                Token::Plus | Token::Minus => {
                    let (_, token) = self.iter.next().unwrap();
                    let right = self.multiplication()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.unary()?;
        while let Some((_, token)) = self.iter.peek() {
            match token {
                Token::Star | Token::Slash => {
                    let (_, token) = self.iter.next().unwrap();
                    let right = self.multiplication()?;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<'a, Expression<'a>> {
        match self.iter.peek() {
            Some((_, Token::Minus)) | Some((_, Token::Bang)) => {
                let (_, token) = self.iter.next().unwrap();
                Ok(Expression::Unary(token, Box::new(self.unary()?)))
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Result<'a, Expression<'a>> {
        let mut expr = self.primary()?;
        if let Some((_, Token::LeftParen)) = self.iter.peek() {
            self.iter.next();
            expr = Expression::Call {
                callee: Box::new(expr),
                args: self.args()?,
            };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<'a, Expression<'a>> {
        match self.iter.next() {
            Some((_, Token::LeftParen)) => {
                let expr = self.expression()?;
                match_token!(self.iter.next(), Token::RightParen, Token::RightParen, {
                    Ok(Expression::Grouping(Box::new(expr)))
                })
            }
            Some((_, Token::Literal(val))) => Ok(Expression::Literal(val)),
            Some((_, ident @ Token::Identifier(_))) => Ok(Expression::Variable(ident)),
            Some((loc, tok)) => Err(ParseError::Unexpected { tok, loc }),
            None => Err(ParseError::EndOfInput),
        }
    }

    fn args(&mut self) -> Result<'a, Vec<Expression<'a>>> {
        let mut vec = vec![self.expression()?];
        loop {
            match self.iter.next() {
                Some((_, Token::RightParen)) => break,
                Some((_, Token::Comma)) => vec.push(self.expression()?),
                Some((loc, found)) => {
                    return Err(ParseError::InvalidToken {
                        expected: Token::Comma,
                        found,
                        loc,
                    })
                }
                None => return Err(ParseError::EndOfInput),
            }
        }
        Ok(vec)
    }
}
