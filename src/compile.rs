use std::convert::TryInto;
use std::iter::Peekable;

use crate::scan::{Token, TokenType, TokenType::*};
use crate::vm::{Instruction, Value};

#[derive(Debug)]
pub enum Error {
    EndOfInput,
    VarStackOverflow,
    Undeclared {
        name: String,
    },
    Mismatch {
        expected: Vec<TokenType>,
        found: Token,
    },
}

type Result<T> = std::result::Result<T, Error>;

struct VarDecl {
    name: String,
    index: u16,
}

pub struct VariableSet {
    decls: Vec<VarDecl>,
    frame: usize,
}

impl VariableSet {
    pub fn new() -> Self {
        VariableSet {
            decls: Vec::new(),
            frame: 0,
        }
    }

    pub fn declare(&mut self, name: String) -> Result<u16> {
        let index: u16 = self
            .decls
            .len()
            .try_into()
            .map_err(|_| Error::VarStackOverflow)?;
        self.decls.push(VarDecl { name, index });
        Ok(index)
    }

    fn find(&self, name: &str) -> Option<&VarDecl> {
        self.decls.iter().rfind(|decl| decl.name == name)
    }
}

pub struct Compiler {
    vars: VariableSet,
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
            vars: VariableSet::new(),
            instrs: Vec::new(),
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instrs
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
            Some(Let) => self.var_decl(it),
            _ => self.expression(it),
        }
    }

    fn emit(&mut self, instr: Instruction) {
        self.instrs.push(instr);
    }

    fn find_var(&self, name: String) -> Result<u16> {
        self.vars
            .find(&name)
            .map(|decl| decl.index)
            .ok_or(Error::Undeclared { name })
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
            _ => self.primary(it)?,
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
                let expected = vec![LeftParen, Identifier(String::new()), Literal(Value::Null)];
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

    fn var_decl<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        it.next(); // Skip Let
        let ident = it.next().ok_or(Error::EndOfInput)?;
        if let Identifier(ident) = ident.ttype {
            let equal = it.next().ok_or(Error::EndOfInput)?;
            if let Equal = equal.ttype {
                self.expression(it)?;
                let idx = self.vars.declare(ident)?;
                self.emit(Instruction::GetLocal(idx));
                Ok(())
            } else {
                let expected = vec![Equal];
                let found = equal;
                Err(Error::Mismatch { expected, found })
            }
        } else {
            let expected = vec![Identifier(String::new())];
            let found = ident;
            Err(Error::Mismatch { expected, found })
        }
    }

    fn variable<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        let token = it.next().unwrap();
        if let Identifier(ident) = token.ttype {
            let idx = self.find_var(ident)?;
            if let Some(Equal) = peek(it) {
                it.next();
                self.expression(it)?;
                self.emit(Instruction::SetLocal(idx));
            } else {
                self.emit(Instruction::GetLocal(idx));
            }
            Ok(())
        } else {
            unreachable!()
        }
    }
}
