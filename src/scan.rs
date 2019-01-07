use crate::value::Value;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Literal(Value),
    Identifier(Cow<'a, str>),
    Let,
    If,
    Else,
    While,
    And,
    Or,
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

impl<'a> Token<'a> {
    pub fn identifier(&self) -> &str {
        match self {
            Token::Identifier(name) => &name,
            x => panic!("Tried to get identifier out of {:?}", x),
        }
    }

    pub fn to_owned(self) -> Self {
        match self {
            Token::Identifier(name) => {
                Token::Identifier(Cow::from(name.into_owned()))
            }
            _ => self
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'a> {
    unread: &'a str,
    loc: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(line: &'a str) -> Lexer<'a> {
        Lexer {
            unread: line,
            loc: 0,
        }
    }

    fn advance(&mut self, count: usize) {
        self.loc = self.loc + count;
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
        self.loc = self.loc + i;
        self.unread = &self.unread[i..];
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

    fn scan(&mut self) -> Option<Token<'a>> {
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
            s.parse::<f64>().ok().map(|x| Token::Literal(Value::Num(x)))
        } else if c.is_alphabetic() {
            let s = self.advance_while(|c| c.is_alphanumeric());
            match s {
                "let" => Some(Token::Let),
                "if" => Some(Token::If),
                "else" => Some(Token::Else),
                "while" => Some(Token::While),
                "and" => Some(Token::And),
                "or" => Some(Token::Or),
                "true" => Some(Token::Literal(Value::Bool(true))),
                "false" => Some(Token::Literal(Value::Bool(true))),
                _ => Some(Token::Identifier(Cow::from(s))),
            }
        } else {
            self.advance(1);
            match c {
                ';' => Some(Token::Semicolon),
                '"' => {
                    let s = self.advance_while(|c| *c != '"');
                    self.advance(1); // Skip trailing quotes
                    Some(Token::Literal(Value::Str(s.to_string())))
                }
                '+' => Some(Token::Plus),
                '-' => Some(Token::Minus),
                '*' => Some(Token::Star),
                '/' => Some(Token::Slash),
                '!' => Some(if self.match_char('=') {
                    Token::BangEqual
                } else {
                    Token::Bang
                }),
                '=' => Some(if self.match_char('=') {
                    Token::EqualEqual
                } else {
                    Token::Equal
                }),
                '>' => Some(if self.match_char('=') {
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }),
                '<' => Some(if self.match_char('=') {
                    Token::LessEqual
                } else {
                    Token::Less
                }),
                '(' => Some(Token::LeftParen),
                ')' => Some(Token::RightParen),
                '[' => Some(Token::LeftBracket),
                ']' => Some(Token::RightBracket),
                '{' => Some(Token::LeftBrace),
                '}' => Some(Token::RightBrace),
                _ => None,
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (usize, Token<'a>);

    fn next(&mut self) -> Option<(usize, Token<'a>)> {
        self.advance_while(|c| c.is_whitespace());
        Some((self.loc + 1, self.scan()?))
    }
}
