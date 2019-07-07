use std::convert::TryInto;
use std::fmt::{self, Display};
use std::iter::Peekable;
use std::num::TryFromIntError;

use crate::scan::{self, Token, TokenType, TokenType::*};
use crate::vm::{Instruction, Value};

#[derive(Debug)]
pub enum Error {
    EndOfInput,
    Scan(scan::Error),
    Conversion(TryFromIntError),
    Mismatch {
        expected: Vec<TokenType>,
        found: Token,
    },
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Conversion(err)
    }
}

impl From<scan::Error> for Error {
    fn from(err: scan::Error) -> Self {
        Error::Scan(err)
    }
}

fn human_readable_fmt<T: Display>(slice: &[T], f: &mut fmt::Formatter) -> fmt::Result {
    match slice.len() {
        0 => write!(f, "nothing"),
        1 => write!(f, "'{}'", slice[0].to_string()),
        x => {
            let mut it = slice[..x - 1].iter();
            write!(f, "one of '{}'", it.next().unwrap())?;
            for val in it {
                write!(f, ", '{}'", val)?;
            }
            write!(f, " or '{}'", slice.last().unwrap())
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EndOfInput => write!(f, "Unexpected end of file"),
            Error::Scan(err) => write!(f, "{}", err),
            Error::Conversion(err) => write!(f, "Number too big to fit into VM code: {}", err),
            Error::Mismatch { expected, found } => {
                write!(f, "Mismatched token: expected ")?;
                human_readable_fmt(&expected, f)?;
                write!(f, ", found '{}'", found.ttype)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Scan(err) => Some(err),
            Error::Conversion(err) => Some(err),
            _ => None,
        }
    }
}

fn unexpected<I>(expected: Vec<TokenType>, it: &mut Peekable<I>) -> Error
where
    I: Iterator<Item = ScanResult>,
{
    match it
        .next()
        .map(|res| res.map_err(Error::Scan))
        .unwrap_or_else(|| Err(Error::EndOfInput))
    {
        Ok(found) => Error::Mismatch { expected, found },
        Err(err) => err,
    }
}

type Result<T> = std::result::Result<T, Error>;

struct VarDecl {
    name: String,
    index: u16,
}

pub struct Compiler {
    locals: Vec<VarDecl>,
    instrs: Vec<Instruction>,
}

type ScanResult = scan::Result<Token>;

fn peek<I>(it: &mut Peekable<I>) -> Result<Option<&TokenType>>
where
    I: Iterator<Item = ScanResult>,
{
    match it.peek() {
        Some(Ok(t)) => Ok(Some(&t.ttype)),
        Some(Err(e)) => Err(Error::Scan(e.clone())),
        None => Ok(None),
    }
}

fn advance<I>(it: &mut Peekable<I>) -> Result<Option<Token>>
where
    I: Iterator<Item = ScanResult>,
{
    it.next().transpose().map_err(Error::Scan)
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            locals: Vec::new(),
            instrs: Vec::new(),
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instrs
    }

    fn emit(&mut self, instr: Instruction) {
        self.instrs.push(instr);
    }

    fn declare_local(&mut self, name: String) -> Result<u16> {
        let index: u16 = self.locals.len().try_into()?;
        self.locals.push(VarDecl { name, index });
        Ok(index)
    }

    fn find_local(&self, name: &str) -> Option<u16> {
        self.locals
            .iter()
            .rfind(|decl| decl.name == name)
            .map(|decl| decl.index)
    }

    fn stub_jump(&mut self) -> usize {
        let idx = self.instrs.len();
        self.emit(Instruction::Pop); // Temporary for jump over if stmt
        idx
    }

    fn patch_jump(
        &mut self,
        src: usize,
        dst: usize,
        f: impl FnOnce(i16) -> Instruction,
    ) -> Result<()> {
        let offset = (dst - src).try_into()?;
        self.instrs[src] = f(offset);
        Ok(())
    }

    pub fn program<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        while let Some(_) = peek(it)? {
            self.declaration(it)?;
            if peek(it)?.is_some() {
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }

    pub fn declaration<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        match peek(it)? {
            Some(Let) => self.local(it),
            Some(Global) => self.global(it),
            _ => self.expression(it),
        }
    }

    fn expression<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.addition(it)
    }

    fn addition<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.multiplication(it)?;
        match peek(it)? {
            Some(Plus) | Some(Minus) => {
                let op = advance(it)?.unwrap();
                self.addition(it)?;
                match op.ttype {
                    Plus => self.emit(Instruction::Add),
                    Minus => self.emit(Instruction::Sub),
                    _ => (),
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn multiplication<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.unary(it)?;
        match peek(it)? {
            Some(Star) | Some(Slash) => {
                let op = advance(it)?.unwrap();
                self.multiplication(it)?;
                match op.ttype {
                    Star => self.emit(Instruction::Mul),
                    Slash => self.emit(Instruction::Div),
                    _ => unreachable!(),
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn unary<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        match peek(it)? {
            Some(Minus) => {
                advance(it)?;
                self.unary(it)?;
                self.emit(Instruction::Neg);
            }
            _ => self.call(it)?,
        }
        Ok(())
    }

    fn call<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.primary(it)?;
        if let Some(LeftParen) = peek(it)? {
            let argc = self.args(it)?;
            self.emit(Instruction::Call(argc));
        }
        Ok(())
    }

    fn primary<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        let token = peek(it)?.ok_or(Error::EndOfInput)?;
        match token {
            LeftParen => self.grouping(it),
            LeftBracket => self.block(it),
            If => self.if_expr(it),
            Function => self.fn_expr(it),
            Identifier(_) => self.variable(it),
            Literal(_) => {
                let token = advance(it)?.unwrap();
                if let Literal(x) = token.ttype {
                    self.emit(Instruction::Push(x));
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            _ => {
                let expected = vec![
                    LeftParen,
                    LeftBracket,
                    If,
                    Function,
                    Identifier(String::new()),
                    Literal(Value::Null),
                ];
                let found = advance(it)?.unwrap();
                Err(Error::Mismatch { expected, found })
            }
        }
    }

    fn grouping<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip LeftParen
        self.expression(it)?;
        match peek(it)? {
            Some(RightParen) => {
                advance(it)?;
                Ok(())
            }
            Some(_) => {
                let found = advance(it)?.unwrap();
                let expected = vec![RightParen];
                Err(Error::Mismatch { expected, found })
            }
            _ => Err(Error::EndOfInput),
        }
    }

    fn block<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip LeftBracket
        let frame_start = self.locals.len();
        loop {
            self.declaration(it)?;
            if let Some(RightBracket) = peek(it)? {
                advance(it)?;
                break;
            } else {
                self.emit(Instruction::Pop);
            }
        }
        let frame_len = self.locals.len() - frame_start;
        self.emit(Instruction::PopFrame(frame_len.try_into()?));
        while self.locals.len() > frame_start {
            self.locals.pop();
        }
        Ok(())
    }

    fn local<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip Let
        let found = advance(it)?.ok_or(Error::EndOfInput)?;
        if let Identifier(ident) = found.ttype {
            let found = advance(it)?.ok_or(Error::EndOfInput)?;
            if let Equal = found.ttype {
                self.expression(it)?;
                let idx = self.declare_local(ident)?;
                self.emit(Instruction::GetLocal(idx));
                Ok(())
            } else {
                let expected = vec![Equal];
                Err(Error::Mismatch { expected, found })
            }
        } else {
            let expected = vec![Identifier(String::new())];
            Err(Error::Mismatch { expected, found })
        }
    }

    fn global<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip Global
        let found = advance(it)?.ok_or(Error::EndOfInput)?;
        if let Identifier(ident) = found.ttype {
            let found = advance(it)?.ok_or(Error::EndOfInput)?;
            if let Equal = found.ttype {
                self.expression(it)?;
                self.emit(Instruction::SetGlobal(ident));
                Ok(())
            } else {
                let expected = vec![Equal];
                Err(Error::Mismatch { expected, found })
            }
        } else {
            let expected = vec![Identifier(String::new())];
            Err(Error::Mismatch { expected, found })
        }
    }

    fn variable<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        let token = advance(it)?.unwrap();
        let follow = peek(it)?;
        match (token.ttype, follow) {
            (Identifier(ident), Some(Equal)) => {
                advance(it)?;
                self.expression(it)?;
                if let Some(idx) = self.find_local(&ident) {
                    self.emit(Instruction::SetLocal(idx));
                } else {
                    self.emit(Instruction::SetGlobal(ident));
                }
                Ok(())
            }
            (Identifier(ident), _) => {
                if let Some(idx) = self.find_local(&ident) {
                    self.emit(Instruction::GetLocal(idx));
                } else {
                    self.emit(Instruction::GetGlobal(ident));
                }
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn if_expr<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip If
        self.expression(it)?; // Condition
        let jump_idx = self.stub_jump();
        match peek(it)? {
            Some(Then) => {
                advance(it)?;
                self.expression(it)?;
            }
            Some(LeftBracket) => {
                self.block(it)?;
            }
            Some(_) => {
                let expected = vec![Then, LeftBracket];
                let found = advance(it)?.unwrap();
                return Err(Error::Mismatch { expected, found });
            }
            None => {
                return Err(Error::EndOfInput);
            }
        };
        let jump_else_idx = self.stub_jump();
        if let Some(Else) = peek(it)? {
            advance(it)?;
            self.expression(it)?;
        } else {
            self.emit(Instruction::Push(Value::Null));
        }
        self.patch_jump(jump_else_idx, self.instrs.len() - 1, Instruction::Jump)?;
        self.patch_jump(jump_idx, jump_else_idx, Instruction::JumpIfZero)?;
        Ok(())
    }

    fn fn_expr<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip Fn

        let mut fn_locals = Vec::new();
        std::mem::swap(&mut self.locals, &mut fn_locals);

        let arity = self.params(it)?;
        let jump_idx = self.stub_jump();
        let code_loc = self.instrs.len();

        match peek(it)? {
            Some(Arrow) => {
                advance(it)?;
                self.expression(it)?;
            }
            Some(LeftBracket) => {
                self.block(it)?;
            }
            Some(_) => {
                let expected = vec![Arrow, LeftBracket];
                let found = advance(it)?.unwrap();
                return Err(Error::Mismatch { expected, found });
            }
            None => {
                return Err(Error::EndOfInput);
            }
        };
        self.emit(Instruction::PopFrame(self.locals.len().try_into()?));
        self.emit(Instruction::Ret);

        self.patch_jump(jump_idx, self.instrs.len() - 1, Instruction::Jump)?;
        self.emit(Instruction::Push(Value::Function { code_loc, arity }));

        std::mem::swap(&mut self.locals, &mut fn_locals);
        Ok(())
    }

    fn params<I>(&mut self, it: &mut Peekable<I>) -> Result<usize>
    where
        I: Iterator<Item = ScanResult>,
    {
        let typ = |t: Token| t.ttype;

        let mut arity = 0;
        if let Some(LeftParen) = advance(it)?.map(typ) {
            match advance(it)?.map(typ) {
                Some(RightParen) => Ok(arity),
                Some(Identifier(a)) => {
                    self.declare_local(a)?;
                    arity = 1;
                    while let Some(Comma) = peek(it)? {
                        advance(it)?;
                        let found = advance(it)?.ok_or(Error::EndOfInput)?;
                        if let Identifier(a) = found.ttype {
                            self.declare_local(a)?;
                            arity += 1;
                        } else {
                            let expected = vec![Identifier(String::new())];
                            return Err(Error::Mismatch { expected, found });
                        }
                    }
                    if let Some(RightParen) = peek(it)? {
                        advance(it)?;
                        Ok(arity)
                    } else {
                        Err(unexpected(vec![RightParen, Comma], it))
                    }
                }
                _ => Err(unexpected(vec![RightParen, Identifier(String::new())], it)),
            }
        } else {
            Err(unexpected(vec![LeftParen], it))
        }
    }

    fn args<I>(&mut self, it: &mut Peekable<I>) -> Result<u16>
    where
        I: Iterator<Item = ScanResult>,
    {
        let mut argc = 0;
        advance(it)?; //Skip LeftParen
        match peek(it)? {
            Some(RightParen) => Ok(argc),
            _ => {
                self.expression(it)?;
                argc += 1;
                while let Some(Comma) = peek(it)? {
                    advance(it)?;
                    self.expression(it)?;
                    argc += 1;
                }
                if let Some(RightParen) = peek(it)? {
                    advance(it)?;
                    Ok(argc)
                } else {
                    Err(unexpected(vec![RightParen, Comma], it))
                }
            }
        }
    }
}
