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
    Minus,
    Plus,
    Slash,
    Star,
    LeftParen,
    RightParen,
    And,
    Or,
    Equal,
    EqualEqual,
}

use TokenType::*;

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

    fn num_literal(&mut self) -> Option<TokenType> {
        let s = self.unread;
        let pos = self.pos;
        self.advance_while(char::is_numeric);
        if let Some('.') = self.peek() {
            self.advance(1);
            self.advance_while(char::is_numeric);
        }
        let len = self.pos - pos;
        let num = s[..len].parse::<f64>().ok()?;
        Some(Literal(Value::Num(num)))
    }

    fn str_literal(&mut self) -> Option<TokenType> {
        let s = self.unread;
        let pos = self.pos;
        self.advance_while(|c| c != '"');
        if let Some('"') = self.peek() {
            let len = self.pos - pos;
            let s = &s[..len];
            self.advance(1);
            Some(Literal(Value::Str(s.to_owned())))
        } else {
            None
        }
    }
}

fn keyword(s: &str) -> Option<TokenType> {
    match s {
        "let" => Some(Let),
        "and" => Some(And),
        "or" => Some(Or),
        "true" => Some(Literal(Value::Bool(true))),
        "false" => Some(Literal(Value::Bool(false))),
        _ => None,
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_while(char::is_whitespace);
        let pos = self.pos;
        let c = self.peek()?;
        let ttype = if c.is_numeric() {
            self.num_literal()
        } else if c.is_alphabetic() || c == '_' {
            let s = self.advance_while(|c| c.is_alphanumeric() || c == '_');
            Some(keyword(s).unwrap_or_else(|| Identifier(s.to_owned())))
        } else {
            self.advance(1);
            match c {
                '"' => self.str_literal(),
                '+' => Some(Plus),
                '-' => Some(Minus),
                '*' => Some(Star),
                '/' => Some(Slash),
                '(' => Some(LeftParen),
                ')' => Some(RightParen),
                '=' => {
                    if let Some('=') = self.peek() {
                        self.advance(1);
                        Some(EqualEqual)
                    } else {
                        Some(Equal)
                    }
                }
                _ => None,
            }
        };
        let len = self.pos - pos;
        let span = Span { pos, len };
        ttype.map(|ttype| Token { ttype, span })
    }
}
