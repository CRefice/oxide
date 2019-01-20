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

    fn token_from(&self, kind: token::Kind<'a>) -> Token<'a> {
        Token { kind, loc: self.loc }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        self.advance_while(|c| c.is_whitespace());
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
            s.parse::<f64>().ok().map(|x| self.token_from(token::Literal(Value::Num(x))))
        } else if c.is_alphabetic() || c == '_' {
            let s = self.advance_while(|c| c.is_alphanumeric() || *c == '_');
            match s {
                "let" => Some(self.token_from(token::Let)),
                "fn" => Some(self.token_from(token::Fn)),
                "if" => Some(self.token_from(token::If)),
                "else" => Some(self.token_from(token::Else)),
                "while" => Some(self.token_from(token::While)),
                "and" => Some(self.token_from(token::And)),
                "or" => Some(self.token_from(token::Or)),
                "return" => Some(self.token_from(token::Return)),
                "true" => Some(self.token_from(token::Literal(Value::Bool(true)))),
                "false" => Some(self.token_from(token::Literal(Value::Bool(true)))),
                "void" => Some(self.token_from(token::Literal(Value::Void))),
                _ => Some(self.token_from(token::Identifier(s))),
            }
        } else {
            self.advance(1);
            match c {
                ',' => Some(self.token_from(token::Comma)),
                ';' => Some(self.token_from(token::Semicolon)),
                '"' => {
                    let s = self.advance_while(|c| *c != '"');
                    self.advance(1); // Skip trailing quotes
                    Some(self.token_from(token::Literal(Value::Str(s.to_string()))))
                }
                '+' => Some(self.token_from(token::Plus)),
                '-' => Some(self.token_from(token::Minus)),
                '*' => Some(self.token_from(token::Star)),
                '/' => {
                    if self.match_char('/') {
                        self.advance_while(|c| *c != '\n');
                        self.next()
                    } else {
                        Some(self.token_from(token::Slash))
                    }
                }
                '!' => Some(if self.match_char('=') {
                    self.token_from(token::BangEqual)
                } else {
                    self.token_from(token::Bang)
                }),
                '=' => Some(if self.match_char('=') {
                    self.token_from(token::EqualEqual)
                } else {
                    self.token_from(token::Equal)
                }),
                '>' => Some(if self.match_char('=') {
                    self.token_from(token::GreaterEqual)
                } else {
                    self.token_from(token::Greater)
                }),
                '<' => Some(if self.match_char('=') {
                    self.token_from(token::LessEqual)
                } else {
                    self.token_from(token::Less)
                }),
                '(' => Some(self.token_from(token::LeftParen)),
                ')' => Some(self.token_from(token::RightParen)),
                '[' => Some(self.token_from(token::LeftBracket)),
                ']' => Some(self.token_from(token::RightBracket)),
                '{' => Some(self.token_from(token::LeftBrace)),
                '}' => Some(self.token_from(token::RightBrace)),
                _ => None,
            }
        }
    }
}
