#[derive(Debug)]
pub enum Token<'a> {
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
    Number(f64),
    StringLiteral(&'a str),
    Identifier(&'a str),
}

pub struct Lexer<'a> {
    unread: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new<S: AsRef<str>>(line: &'a S) -> Lexer<'a> {
        Lexer {
            unread: line.as_ref(),
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
            Some(Token::Identifier(s))
        } else {
            self.unread = &self.unread[1..];
            match c {
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
