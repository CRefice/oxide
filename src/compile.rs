use std::convert::TryInto;
use std::iter::Peekable;
use std::num::TryFromIntError;

use crate::scan::{Token, TokenType, TokenType::*};
use crate::vm::{Instruction, Value};

#[derive(Debug)]
pub enum Error {
    EndOfInput,
    IntConversion,
    Mismatch {
        expected: Vec<TokenType>,
        found: Token,
    },
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Error::IntConversion
    }
}

fn unexpected<I>(expected: Vec<TokenType>, it: &mut Peekable<I>) -> Error
where
    I: Iterator<Item = Token>,
{
    it.next()
        .map(|found| Error::Mismatch { expected, found })
        .unwrap_or_else(|| Error::EndOfInput)
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

fn peek<I>(it: &mut Peekable<I>) -> Option<&TokenType>
where
    I: Iterator<Item = Token>,
{
    it.peek().map(|t| &t.ttype)
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
        I: Iterator<Item = Token>,
    {
        while let Some(_) = peek(it) {
            self.declaration(it)?;
            if peek(it).is_some() {
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }

    pub fn declaration<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        match peek(it) {
            Some(Let) => self.local(it),
            Some(Global) => self.global(it),
            _ => self.expression(it),
        }
    }

    fn expression<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        self.addition(it)
    }

    fn addition<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        self.multiplication(it)?;
        match peek(it) {
            Some(Plus) | Some(Minus) => {
                let op = it.next().unwrap();
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
        I: Iterator<Item = Token>,
    {
        self.unary(it)?;
        match peek(it) {
            Some(Star) | Some(Slash) => {
                let op = it.next().unwrap();
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
        I: Iterator<Item = Token>,
    {
        match peek(it) {
            Some(Minus) => {
                it.next();
                self.unary(it)?;
                self.emit(Instruction::Neg);
            }
            _ => self.call(it)?,
        }
        Ok(())
    }

    fn call<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        let begin = self.instrs.len();
        self.primary(it)?;
        let end = self.instrs.len();
        if let Some(LeftParen) = peek(it) {
            let argc = self.args(it)?;
            // Since we're compiling in one pass, we need to
            // move callable expression to after args
            let mut expr = self.instrs.drain(begin..end).collect();
            self.instrs.append(&mut expr);
            self.emit(Instruction::Call(argc));
        }
        Ok(())
    }

    fn primary<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        let token = peek(it).ok_or(Error::EndOfInput)?;
        match token {
            LeftParen => self.grouping(it),
            LeftBracket => self.block(it),
            If => self.if_expr(it),
            Function => self.fn_expr(it),
            Identifier(_) => self.variable(it),
            Literal(_) => {
                let token = it.next().unwrap();
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
                let found = it.next().unwrap();
                Err(Error::Mismatch { expected, found })
            }
        }
    }

    fn grouping<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip LeftParen
        self.expression(it)?;
        match peek(it) {
            Some(RightParen) => {
                it.next();
                Ok(())
            }
            Some(_) => {
                let found = it.next().unwrap();
                let expected = vec![RightParen];
                Err(Error::Mismatch { expected, found })
            }
            _ => Err(Error::EndOfInput),
        }
    }

    fn block<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip LeftBracket
        let frame_start = self.locals.len();
        loop {
            self.declaration(it)?;
            if let Some(RightBracket) = peek(it) {
                it.next();
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
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip Let
        let found = it.next().ok_or(Error::EndOfInput)?;
        if let Identifier(ident) = found.ttype {
            let found = it.next().ok_or(Error::EndOfInput)?;
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
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip Global
        let found = it.next().ok_or(Error::EndOfInput)?;
        if let Identifier(ident) = found.ttype {
            let found = it.next().ok_or(Error::EndOfInput)?;
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
        I: Iterator<Item = Token>,
    {
        let token = it.next().unwrap();
        let follow = peek(it);
        match (token.ttype, follow) {
            (Identifier(ident), Some(Equal)) => {
                it.next();
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
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip If
        self.expression(it)?; // Condition
        let jump_idx = self.stub_jump();
        match peek(it) {
            Some(Then) => {
                it.next();
                self.expression(it)?;
            }
            Some(LeftBracket) => {
                self.block(it)?;
            }
            Some(_) => {
                let expected = vec![Then, LeftBracket];
                let found = it.next().unwrap();
                return Err(Error::Mismatch { expected, found });
            }
            None => {
                return Err(Error::EndOfInput);
            }
        };
        let jump_else_idx = self.stub_jump();
        if let Some(Else) = peek(it) {
            it.next();
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
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip Fn

        let mut fn_locals = Vec::new();
        std::mem::swap(&mut self.locals, &mut fn_locals);

        let arity = self.params(it)?;
        let code_loc = self.instrs.len();
        let jump_idx = self.stub_jump();

        match peek(it) {
            Some(Arrow) => {
                it.next();
                self.expression(it)?;
            }
            Some(LeftBracket) => {
                self.block(it)?;
            }
            Some(_) => {
                let expected = vec![Then, LeftBracket];
                let found = it.next().unwrap();
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
        I: Iterator<Item = Token>,
    {
        let typ = |t: Token| t.ttype;

        let mut arity = 0;
        if let Some(LeftParen) = it.next().map(typ) {
            match it.next().map(typ) {
                Some(RightParen) => Ok(arity),
                Some(Identifier(a)) => {
                    self.declare_local(a)?;
                    arity = 1;
                    while let Some(Comma) = peek(it) {
                        it.next();
                        let found = it.next().ok_or(Error::EndOfInput)?;
                        if let Identifier(a) = found.ttype {
                            self.declare_local(a)?;
                            arity += 1;
                        } else {
                            let expected = vec![Identifier(String::new())];
                            return Err(Error::Mismatch { expected, found });
                        }
                    }
                    if let Some(RightParen) = peek(it) {
                        it.next();
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
        I: Iterator<Item = Token>,
    {
        let mut argc = 0;
        it.next(); //Skip LeftParen
        match peek(it) {
            Some(RightParen) => Ok(argc),
            _ => {
                self.expression(it)?;
                argc += 1;
                while let Some(Comma) = peek(it) {
                    it.next();
                    self.expression(it)?;
                    argc += 1;
                }
                if let Some(RightParen) = peek(it) {
                    it.next();
                    Ok(argc)
                } else {
                    Err(unexpected(vec![RightParen, Comma], it))
                }
            }
        }
    }
}
