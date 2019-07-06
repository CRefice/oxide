mod value;

use std::collections::HashMap;
use std::convert::TryInto as _;
use std::num::TryFromIntError;

pub use value::Value;

#[derive(Debug)]
pub enum Instruction {
    Push(Value),
    GetLocal(u16),
    SetLocal(u16),
    GetGlobal(String),
    SetGlobal(String),
    Pop,
    PopFrame(u16),
    Jump(i16),
    JumpIfZero(i16),
    Call(u16),
    Ret,
    Add,
    Sub,
    Mul,
    Div,
    Neg,
}

#[derive(Debug)]
pub enum Error {
    Value(value::Error),
    Conversion(TryFromIntError),
    UndeclaredGlobal(String),
    WrongArgCount { expected: usize, found: u16 },
    EmptyStack,
}

impl From<value::Error> for Error {
    fn from(err: value::Error) -> Self {
        Error::Value(err)
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Conversion(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Frame {
    call_site: usize,
    stack_depth: usize,
}

pub struct VirtualMachine {
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    ip: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            globals: HashMap::new(),
            stack: Vec::new(),
            frames: Vec::new(),
            ip: 0,
        }
    }

    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::EmptyStack)
    }

    pub fn peek(&mut self) -> Result<Value> {
        self.stack.last().cloned().ok_or(Error::EmptyStack)
    }

    pub fn define(&mut self, name: String, val: Value) {
        self.globals.insert(name, val);
    }

    fn jump(&mut self, offset: i16) -> Result<()> {
        let mut ip: isize = self.ip.try_into()?;
        ip += isize::from(offset);
        self.ip = ip.try_into()?;
        Ok(())
    }

    fn local_idx(&mut self, offset: u16) -> usize {
        let frame_idx = self.frames.last().map(|f| f.stack_depth).unwrap_or(0);
        usize::from(offset) + frame_idx
    }

    pub fn step(&mut self, code: &[Instruction]) -> Result<()> {
        let result = match &code[self.ip] {
            Instruction::Push(val) => {
                self.stack.push(val.clone());
                Ok(())
            }
            Instruction::Pop => self.pop().map(|_| ()),
            Instruction::PopFrame(n) => {
                let top = self.pop()?;
                for _ in 0..*n {
                    self.pop()?;
                }
                self.stack.push(top);
                Ok(())
            }
            Instruction::GetGlobal(name) => {
                let val = self
                    .globals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Error::UndeclaredGlobal(name.clone()))?;
                self.stack.push(val);
                Ok(())
            }
            Instruction::SetGlobal(name) => {
                let val = self.peek()?;
                self.globals.insert(name.clone(), val);
                Ok(())
            }
            Instruction::GetLocal(idx) => {
                let idx = self.local_idx(*idx);
                let val = self
                    .stack
                    .get(idx)
                    .cloned()
                    .expect("Tried to get nonexistent variable!");
                self.stack.push(val);
                Ok(())
            }
            Instruction::SetLocal(idx) => {
                let val = self.peek()?;
                let idx = self.local_idx(*idx);
                self.stack[idx] = val;
                Ok(())
            }
            Instruction::Jump(offset) => self.jump(*offset),
            Instruction::JumpIfZero(offset) => {
                let cond = self.pop()?;
                if !cond.is_truthy() {
                    self.jump(*offset)?;
                }
                Ok(())
            }
            Instruction::Call(argc) => {
                let callable = self.pop()?;
                match callable {
                    Value::Function { code_loc, arity } => {
                        if usize::from(*argc) == arity {
                            let frame = Frame {
                                call_site: self.ip,
                                stack_depth: self.stack.len() - arity,
                            };
                            self.frames.push(frame);
                            self.ip = code_loc;
                            Ok(())
                        } else {
                            Err(Error::WrongArgCount {
                                expected: arity,
                                found: *argc,
                            })
                        }
                    }
                    Value::NativeFn { f, arity } => {
                        let begin = self.stack.len() - arity;
                        let result = f(&self.stack[begin..]);
                        self.stack.drain(begin..);
                        self.stack.push(result);
                        Ok(())
                    }
                    _ => Err(Error::Value(value::Error::WrongCall(callable))),
                }
            }
            Instruction::Ret => {
                let frame = self.frames.pop().ok_or(Error::EmptyStack)?;
                self.ip = frame.call_site;
                Ok(())
            }
            Instruction::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = (a + b)?;
                self.stack.push(result);
                Ok(())
            }
            Instruction::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = (a - b)?;
                self.stack.push(result);
                Ok(())
            }
            Instruction::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = (a * b)?;
                self.stack.push(result);
                Ok(())
            }
            Instruction::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = (a / b)?;
                self.stack.push(result);
                Ok(())
            }
            Instruction::Neg => {
                let a = self.pop()?;
                let result = (-a)?;
                self.stack.push(result);
                Ok(())
            }
        };
        self.ip += 1;
        result
    }

    pub fn run(&mut self, code: &[Instruction]) -> Result<()> {
        while self.ip < code.len() {
            self.step(code)?;
        }
        Ok(())
    }
}
