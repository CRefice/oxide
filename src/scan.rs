use crate::vm::Value;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug)]
pub enum TokenType {
    Literal(Value),
    Minus,
    Plus,
    Slash,
    Star,
    LeftParen,
    RightParen,
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
            .unwrap_or(self.unread.len());
        self.advance(i)
    }

    fn num_literal(&mut self) -> Option<Token> {
        let s = self.unread;
        let pos = self.pos;
        self.advance_while(char::is_numeric);
        if let Some('.') = self.peek() {
            self.advance(1);
            self.advance_while(char::is_numeric);
        }
        let len = self.pos - pos;
        let num = s[..len].parse::<f64>().ok()?;
        let token = Token {
            ttype: Literal(Value::Num(num)),
            span: Span { pos, len },
        };
        Some(token)
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_while(char::is_whitespace);
        let c = self.peek()?;
        if c.is_numeric() {
            self.num_literal()
        } else {
            let pos = self.pos;
            self.advance(1);
            let ttype = match c {
                '+' => Some(Plus),
                '-' => Some(Minus),
                '*' => Some(Star),
                '/' => Some(Slash),
                '(' => Some(LeftParen),
                ')' => Some(RightParen),
                _ => None,
            };;
            let len = self.pos - pos;
            let span = Span { pos, len };
            ttype.map(|ttype| Token { ttype, span })
        }
    }
}
