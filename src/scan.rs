use std::fmt::{self, Display};
use std::num::ParseFloatError;

use crate::vm::Value;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug)]
pub enum TokenType {
    Literal(Value),
    Identifier(String),
    Let,
    Global,
    If,
    Then,
    Else,
    Function,
    Minus,
    Plus,
    Slash,
    Star,
    Arrow,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    And,
    Or,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Not,
    Comma,
}

use TokenType::*;

impl Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Literal(_) => "literal",
                Identifier(_) => "identifier",
                Let => "let",
                Global => "global",
                If => "if",
                Then => "then",
                Else => "else",
                Function => "fn",
                Minus => "-",
                Plus => "+",
                Slash => "/",
                Star => "*",
                Arrow => "=>",
                LeftParen => "(",
                RightParen => ")",
                LeftBracket => "{",
                RightBracket => "}",
                And => "and",
                Or => "or",
                Not => "not",
                Equal => "=",
                EqualEqual => "==",
                Bang => "!",
                BangEqual => "!=",
                Greater => ">",
                GreaterEqual => ">=",
                Less => "<",
                LessEqual => "<=",
                Comma => ",",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    UnclosedQuote { span: Span },
    ParseNum { cause: ParseFloatError, span: Span },
    Unrecognized { c: char, span: Span },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnclosedQuote { .. } => write!(f, "Unmatched opening quote"),
            Error::ParseNum { cause, .. } => write!(f, "Unable to parse number: {}", cause),
            Error::Unrecognized { c, .. } => write!(f, "Invalid token '{}'", c),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ParseNum { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub span: Span,
}

pub struct Scanner<'a> {
    unread: &'a str,
    pos: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Self {
        Scanner { unread: s, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.unread.chars().next()
    }

    fn advance(&mut self, cnt: usize) -> &str {
        let s = &self.unread[..cnt];
        self.unread = &self.unread[cnt..];
        self.pos += cnt;
        s
    }

    fn advance_while<F>(&mut self, pattern: F) -> &str
    where
        F: Fn(char) -> bool,
    {
        let i = self
            .unread
            .char_indices()
            .skip_while(|(_, c)| pattern(*c))
            .map(|x| x.0)
            .next()
            .unwrap_or_else(|| self.unread.len());
        self.advance(i)
    }

    fn num_literal(&mut self) -> Result<TokenType> {
        let s = self.unread;
        let pos = self.pos;
        self.advance_while(char::is_numeric);
        if let Some('.') = self.peek() {
            self.advance(1);
            self.advance_while(char::is_numeric);
        }
        let len = self.pos - pos;
        let span = Span { pos, len };
        let num = s[..len]
            .parse::<f64>()
            .map_err(|cause| Error::ParseNum { cause, span })?;
        Ok(Literal(Value::Num(num)))
    }

    fn str_literal(&mut self) -> Result<TokenType> {
        let s = self.unread;
        let pos = self.pos;
        self.advance_while(|c| c != '"');
        let len = self.pos - pos;
        if let Some('"') = self.peek() {
            let s = &s[..len];
            self.advance(1);
            Ok(Literal(Value::Str(s.to_owned())))
        } else {
            let span = Span { pos, len };
            Err(Error::UnclosedQuote { span })
        }
    }
}

fn keyword(s: &str) -> Option<TokenType> {
    match s {
        "let" => Some(Let),
        "global" => Some(Global),
        "if" => Some(If),
        "then" => Some(Then),
        "else" => Some(Else),
        "fn" => Some(Function),
        "and" => Some(And),
        "or" => Some(Or),
        "not" => Some(Not),
        "true" => Some(Literal(Value::Bool(true))),
        "false" => Some(Literal(Value::Bool(false))),
        "null" => Some(Literal(Value::Null)),
        _ => None,
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_while(char::is_whitespace);
        let pos = self.pos;
        let c = self.peek()?;
        let result = if c.is_numeric() {
            self.num_literal()
        } else if c.is_alphabetic() || c == '_' {
            let s = self.advance_while(|c| c.is_alphanumeric() || c == '_');
            Ok(keyword(s).unwrap_or_else(|| Identifier(s.to_owned())))
        } else {
            self.advance(1);
            match c {
                '"' => self.str_literal(),
                '+' => Ok(Plus),
                ',' => Ok(Comma),
                '-' => Ok(Minus),
                '*' => Ok(Star),
                '/' => Ok(Slash),
                '(' => Ok(LeftParen),
                ')' => Ok(RightParen),
                '{' => Ok(LeftBracket),
                '}' => Ok(RightBracket),
                '=' => match self.peek() {
                    Some('=') => {
                        self.advance(1);
                        Ok(EqualEqual)
                    }
                    Some('>') => {
                        self.advance(1);
                        Ok(Arrow)
                    }
                    _ => Ok(Equal),
                },
                '!' => match self.peek() {
                    Some('=') => {
                        self.advance(1);
                        Ok(BangEqual)
                    }
                    _ => Ok(Bang),
                },
                '>' => match self.peek() {
                    Some('=') => {
                        self.advance(1);
                        Ok(GreaterEqual)
                    }
                    _ => Ok(Greater),
                },
                '<' => match self.peek() {
                    Some('=') => {
                        self.advance(1);
                        Ok(LessEqual)
                    }
                    _ => Ok(Less),
                },
                c => {
                    let len = self.pos - pos;
                    let span = Span { pos, len };
                    Err(Error::Unrecognized { c, span })
                }
            }
        };
        let len = self.pos - pos;
        let span = Span { pos, len };
        Some(result.map(|ttype| Token { ttype, span }))
    }
}
