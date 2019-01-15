use self::Kind::*;
use crate::value::Value;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Kind<'a> {
    Literal(Value<'a>),
    Identifier(&'a str),
    Let,
    Fn,
    If,
    Else,
    While,
    And,
    Or,
    Return,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    Equal,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

pub use self::Kind::*;

impl<'a> Display for Kind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Literal(_) => "literal",
                Identifier(_) => "identifier",
                Let => "'let'",
                Fn => "'fn'",
                If => "'if'",
                Else => "'else'",
                While => "'while'",
                And => "'and'",
                Or => "'or'",
                Return => "'return'",
                Comma => "','",
                Semicolon => "';'",
                Plus => "'+'",
                Minus => "'-'",
                Star => "'*'",
                Slash => "'/'",
                Bang => "'!'",
                Equal => "'='",
                EqualEqual => "'=='",
                BangEqual => "'!='",
                Greater => "'>'",
                GreaterEqual => "'>='",
                Less => "'<'",
                LessEqual => "'<='",
                LeftParen => "'('",
                RightParen => "')'",
                LeftBracket => "'['",
                RightBracket => "']'",
                LeftBrace => "'{'",
                RightBrace => "'}'",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: Kind<'a>,
    pub loc: usize,
}

impl<'a> Token<'a> {
    pub fn identifier(&self) -> &str {
        match &self.kind {
            Kind::Identifier(name) => name,
            x => panic!("Tried to get identifier out of {}", x),
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}
