use crate::scan::{Token, TokenType, TokenType::*};
use crate::vm::Instruction;

/*
pub enum ExprType {
    Literal(Value),
    Binary {
        a: Box<Expression>,
        b: Box<Expression>,
        op: Token,
    },
}

*/

#[derive(Debug)]
pub enum Error {
    A,
}

type Result<T> = std::result::Result<T, Error>;

pub struct Parser<I: Iterator<Item = Token>> {
    it: std::iter::Peekable<I>,
    instrs: Vec<Instruction>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(i: I) -> Self {
        Parser {
            it: i.peekable(),
            instrs: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Result<Vec<Instruction>> {
        self.expression()?;
        Ok(self.instrs)
    }

    fn peek(&mut self) -> Option<&TokenType> {
        self.it.peek().map(|t| &t.ttype)
    }

    fn emit(&mut self, instr: Instruction) {
        self.instrs.push(instr);
    }

    fn expression(&mut self) -> Result<()> {
        self.addition()
    }

    fn addition(&mut self) -> Result<()> {
        self.multiplication()?;
        match self.peek() {
            Some(Plus) | Some(Minus) => {
                let op = self.it.next().unwrap();
                self.addition()?;
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

    fn multiplication(&mut self) -> Result<()> {
        self.unary()?;
        match self.peek() {
            Some(Star) | Some(Slash) => {
                let op = self.it.next().unwrap();
                self.multiplication()?;
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

    fn unary(&mut self) -> Result<()> {
        match self.peek() {
            Some(Minus) => {
                self.it.next();
                self.unary()?;
                self.emit(Instruction::Neg);
            }
            _ => self.primary()?,
        }
        Ok(())
    }

    fn primary(&mut self) -> Result<()> {
        let token = self.it.next().ok_or(Error::A)?;
        match token.ttype {
            LeftParen => {
                self.it.next();
                self.expression()?;
                if let Some(RightParen) = self.peek() {
                    self.it.next();
                    Ok(())
                } else {
                    Err(Error::A)
                }
            }
            Literal(x) => {
                self.emit(Instruction::Push(x));
                Ok(())
            }
            _ => Err(Error::A),
        }
    }
}
