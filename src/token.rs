#[derive(Debug)]
pub enum Token<'a> {
    Number(f64),
    Bool(bool),
    StringLiteral(&'a str),
    Identifier(&'a str),
    Let,
    If,
    Else,
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

#[derive(Clone)]
pub struct Lexer<'a> {
    unread: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(line: &'a str) -> Lexer<'a> {
        Lexer {
            unread: line,
        }
    }

    fn advance_while<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(&char) -> bool,
    {
        let cs = self.unread.char_indices();
        let mut cs = cs.skip_while(|(_, c)| predicate(c));
        let (i, _) = cs.next().unwrap_or((0, '\0'));
        let s = &self.unread[..i];
        self.unread = &self.unread[i..];
        s
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        self.unread = self.unread.trim_start();
        let c = self.unread.chars().next()?;
        if c.is_numeric() {
            let s = self.advance_while(|c| c.is_numeric());
            s.parse::<f64>().ok().map(|x| Token::Number(x))
        } else if c.is_alphabetic() {
            let s = self.advance_while(|c| c.is_alphanumeric());
            match s {
                "let" => Some(Token::Let),
                "if" => Some(Token::If),
                "else" => Some(Token::Else),
                "true" => Some(Token::Bool(true)),
                "false" => Some(Token::Bool(false)),
                _ => Some(Token::Identifier(s))
            }
        } else {
            self.unread = &self.unread[1..];
            match c {
                ';' => Some(Token::Semicolon),
                '"' => {
                    let s = self.advance_while(|c| *c != '"');
                    self.unread = &self.unread[1..]; // Skip trailing quotes
                    Some(Token::StringLiteral(s))
                }
                '+' => Some(Token::Plus),
                '-' => Some(Token::Minus),
                '*' => Some(Token::Star),
                '/' => Some(Token::Slash),
                '!' => Some(self.unread.chars().next().map_or(Token::Bang, |c| {
                    if c == '=' {
                        self.unread = &self.unread[1..];
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                })),
                '=' => Some(self.unread.chars().next().map_or(Token::Equal, |c| {
                    if c == '=' {
                        self.unread = &self.unread[1..];
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                })),
                '>' => Some(self.unread.chars().next().map_or(Token::Greater, |c| {
                    if c == '=' {
                        self.unread = &self.unread[1..];
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                })),
                '<' => Some(self.unread.chars().next().map_or(Token::Less, |c| {
                    if c == '=' {
                        self.unread = &self.unread[1..];
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                })),
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
