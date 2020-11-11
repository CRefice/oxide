use std::fmt::{self, Display};
use std::num::ParseFloatError;

use crate::loc::{Locate, SourceLocation};
use crate::vm::Value;

#[derive(Debug)]
pub enum TokenType {
    Literal(Value),
    Identifier(String),
    Let,
    Global,
    If,
    Then,
    Else,
    While,
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
                While => "while",
                Function => "fn",
                Minus => "-",
                Plus => "+",
                Slash => "/",
                Star => "*",
                Arrow => "->",
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

#[derive(Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub loc: SourceLocation,
}

pub struct TokenStream<'a> {
    unread: &'a str,
    pos: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(s: &'a str) -> Self {
        TokenStream { unread: s, pos: 0 }
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

    fn num_literal(&mut self) -> std::result::Result<TokenType, ErrorKind> {
        let s = self.unread;
        let offset = self.pos;
        self.advance_while(char::is_numeric);
        if let Some('.') = self.peek() {
            self.advance(1);
            self.advance_while(char::is_numeric);
        }
        let len = self.pos - offset;
        s[..len]
            .parse::<f64>()
            .map(|num| Literal(Value::Num(num)))
            .map_err(ErrorKind::ParseNum)
    }

    fn str_literal(&mut self) -> std::result::Result<TokenType, ErrorKind> {
        let s = self.unread;
        let offset = self.pos;
        self.advance_while(|c| c != '"');
        if let Some('"') = self.peek() {
            let len = self.pos - offset;
            self.advance(1);
            let s = &s[..len];
            Ok(Literal(Value::Str(s.to_owned())))
        } else {
            Err(ErrorKind::UnmatchedQuote)
        }
    }

    fn skip_block_comment(&mut self) -> std::result::Result<(), ErrorKind> {
        loop {
            self.advance_while(|c| c != '*');
            match self.peek() {
                Some(x) => {
                    self.advance(1);
                    if x == '/' {
                        self.advance(1);
                        return Ok(());
                    }
                }
                None => {
                    return Err(ErrorKind::UnmatchedComment);
                }
            }
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
        "while" => Some(While),
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

impl<'a> Iterator for TokenStream<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_while(char::is_whitespace);
        let offset = self.pos;
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
                '-' => match self.peek() {
                    Some('>') => {
                        self.advance(1);
                        Ok(Arrow)
                    }
                    _ => Ok(Minus),
                },
                '*' => Ok(Star),
                '/' => match self.peek() {
                    Some('/') => {
                        self.advance_while(|c| c != '\n');
                        return self.next();
                    }
                    Some('*') => {
                        self.advance(1);
                        match self.skip_block_comment() {
                            Ok(()) => return self.next(),
                            Err(e) => Err(e),
                        }
                    }
                    _ => Ok(Slash),
                },
                '(' => Ok(LeftParen),
                ')' => Ok(RightParen),
                '{' => Ok(LeftBracket),
                '}' => Ok(RightBracket),
                '=' => match self.peek() {
                    Some('=') => {
                        self.advance(1);
                        Ok(EqualEqual)
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
                c => Err(ErrorKind::Unrecognized(c)),
            }
        };
        let len = self.pos - offset;
        let loc = SourceLocation { offset, len };
        Some(
            result
                .map(|ttype| Token { ttype, loc })
                .map_err(|kind| Error { kind, loc }),
        )
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnmatchedQuote,
    UnmatchedComment,
    ParseNum(ParseFloatError),
    Unrecognized(char),
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    loc: SourceLocation,
}

impl Locate for Error {
    fn location(&self) -> SourceLocation {
        self.loc
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnmatchedQuote => write!(f, "Unmatched quote"),
            ErrorKind::UnmatchedComment => write!(f, "Unterminated block comment"),
            ErrorKind::ParseNum(cause) => write!(f, "Unable to parse number: {}", cause),
            ErrorKind::Unrecognized(c) => write!(f, "Invalid token '{}'", c),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::ParseNum(cause) => Some(cause),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
