use super::{token, Token};
use crate::value::Value;

#[derive(Clone)]
pub struct Lexer<'a> {
    unread: &'a str,
    loc: (usize, usize),
}

impl<'a> Lexer<'a> {
    pub fn new(line: &'a str) -> Lexer<'a> {
        Lexer {
            unread: line,
            loc: (1, 1),
        }
    }

    fn advance(&mut self, count: usize) {
        let (ref mut row, ref mut col) = self.loc;
        for c in self.unread[..count].chars() {
            if c == '\n' {
                *row = *row + 1;
                *col = 1;
            } else {
                *col = *col + 1;
            }
        }
        self.unread = &self.unread[count..];
    }

    fn advance_while<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(&char) -> bool,
    {
        let cs = self.unread.char_indices();
        let mut cs = cs.skip_while(|(_, c)| predicate(c));
        let (i, _) = cs.next().unwrap_or((0, '\0'));
        let s = &self.unread[..i];
        self.advance(i);
        s
    }

    fn match_char(&mut self, c: char) -> bool {
        self.unread
            .chars()
            .next()
            .map(|d| {
                let ret = d == c;
                if ret {
                    self.advance(1);
                }
                ret
            })
            .unwrap_or(false)
    }

    fn scan(&mut self) -> Option<token::Kind<'a>> {
        let c = self.unread.chars().next()?;
        if c.is_numeric() {
            let cs = self.unread.char_indices();
            let mut cs = cs.skip_while(|(_, c)| c.is_numeric());
            let (mut i, _) = cs.next().unwrap_or((0, '\0'));
            if let Some((_, '.')) = cs.next() {
                let mut cs = cs.skip_while(|(_, c)| c.is_numeric());
                let (idx, _) = cs.next().unwrap_or((0, '\0'));
                i = idx;
            }
            let s = &self.unread[..i];
            self.advance(i);
            s.parse::<f64>().ok().map(|x| token::Literal(Value::Num(x)))
        } else if c.is_alphabetic() || c == '_' {
            let s = self.advance_while(|c| c.is_alphanumeric() || *c == '_');
            match s {
                "let" => Some(token::Let),
                "fn" => Some(token::Fn),
                "if" => Some(token::If),
                "else" => Some(token::Else),
                "while" => Some(token::While),
                "and" => Some(token::And),
                "or" => Some(token::Or),
                "return" => Some(token::Return),
                "true" => Some(token::Literal(Value::Bool(true))),
                "false" => Some(token::Literal(Value::Bool(true))),
                _ => Some(token::Identifier(s)),
            }
        } else {
            self.advance(1);
            match c {
                ',' => Some(token::Comma),
                ';' => Some(token::Semicolon),
                '"' => {
                    let s = self.advance_while(|c| *c != '"');
                    self.advance(1); // Skip trailing quotes
                    Some(token::Literal(Value::Str(s.to_string())))
                }
                '+' => Some(token::Plus),
                '-' => Some(token::Minus),
                '*' => Some(token::Star),
                '/' => Some(token::Slash),
                '!' => Some(if self.match_char('=') {
                    token::BangEqual
                } else {
                    token::Bang
                }),
                '=' => Some(if self.match_char('=') {
                    token::EqualEqual
                } else {
                    token::Equal
                }),
                '>' => Some(if self.match_char('=') {
                    token::GreaterEqual
                } else {
                    token::Greater
                }),
                '<' => Some(if self.match_char('=') {
                    token::LessEqual
                } else {
                    token::Less
                }),
                '(' => Some(token::LeftParen),
                ')' => Some(token::RightParen),
                '[' => Some(token::LeftBracket),
                ']' => Some(token::RightBracket),
                '{' => Some(token::LeftBrace),
                '}' => Some(token::RightBrace),
                _ => None,
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        self.advance_while(|c| c.is_whitespace());
        let loc = self.loc;
        self.scan().map(|kind| Token { kind, loc })
    }
}
