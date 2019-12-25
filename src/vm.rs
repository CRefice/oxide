mod value;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto as _;
use std::fmt::{self, Display};
use std::num::TryFromIntError;
use std::rc::Rc;

pub use value::Value;

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    GetLocal(u16),
    SetLocal(u16),
    GetGlobal(String),
    SetGlobal(String),
    Pop,
    // Dumb hacks
    SaveReturn,
    RestoreReturn,
    Jump(i16),
    JumpIfFalse(i16),
    JumpIfTrue(i16),
    Call(u16),
    Ret,
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Not,
    Equal,
    Less,
    Greater,
    Temp, // Panics if encountered in code
}

pub type Chunk = Rc<Vec<Instruction>>;

#[derive(Debug, Clone)]
pub struct CodeLocation {
    chunk: Chunk,
    ip: usize,
}

impl CodeLocation {
    pub fn new(chunk: Chunk) -> Self {
        CodeLocation { chunk, ip: 0 }
    }

    pub fn is_at_end(&self) -> bool {
        self.ip == self.chunk.len()
    }

    pub fn jump(&mut self, offset: i16) -> Result<()> {
        let mut ip: isize = self.ip.try_into()?;
        ip += isize::from(offset);
        self.ip = ip.try_into()?;
        Ok(())
    }
}

#[derive(Debug)]
struct Frame {
    call_loc: CodeLocation,
    stack_depth: usize,
}

pub struct VirtualMachine {
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    ret_channel: Option<Value>,
    frames: Vec<Frame>,
    loc: CodeLocation,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        VirtualMachine {
            globals: HashMap::new(),
            stack: vec![Value::Null],
            ret_channel: None,
            frames: Vec::new(),
            loc: CodeLocation::new(chunk),
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

    fn local_idx(&mut self, offset: u16) -> usize {
        let frame_idx = self.frames.last().map(|f| f.stack_depth).unwrap_or(0);
        usize::from(offset) + frame_idx
    }

    fn step(&mut self) -> Result<()> {
        let opcode = self.loc.chunk[self.loc.ip].clone();
        self.loc.ip += 1;
        match opcode {
            Instruction::Push(val) => {
                self.stack.push(val);
                Ok(())
            }
            Instruction::Pop => self.pop().map(|_| ()),
            Instruction::SaveReturn => {
                let top = self.pop()?;
                self.ret_channel.replace(top);
                Ok(())
            }
            Instruction::RestoreReturn => {
                let ret = self.ret_channel.take();
                let ret_val = ret.ok_or_else(|| Error::NoReturnValue)?;
                self.stack.push(ret_val);
                Ok(())
            }
            Instruction::GetGlobal(name) => {
                let val = self
                    .globals
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| Error::UndeclaredGlobal(name.clone()))?;
                self.stack.push(val);
                Ok(())
            }
            Instruction::SetGlobal(name) => {
                let val = self.peek()?;
                self.globals.insert(name, val);
                Ok(())
            }
            Instruction::GetLocal(idx) => {
                let idx = self.local_idx(idx);
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
                let idx = self.local_idx(idx);
                self.stack[idx] = val;
                Ok(())
            }
            Instruction::Jump(offset) => self.loc.jump(offset),
            Instruction::JumpIfFalse(offset) => {
                let cond = self.peek()?;
                if !cond.is_truthy() {
                    self.loc.jump(offset)?;
                }
                Ok(())
            }
            Instruction::JumpIfTrue(offset) => {
                let cond = self.peek()?;
                if cond.is_truthy() {
                    self.loc.jump(offset)?;
                }
                Ok(())
            }
            Instruction::Call(argc) => {
                let argn = usize::from(argc);
                let index = self.stack.len() - argn - 1;
                let callable = &self.stack[index];
                match callable {
                    Value::Function { chunk, arity, .. } => {
                        if &argn == arity {
                            let frame = Frame {
                                call_loc: self.loc.clone(),
                                stack_depth: self.stack.len() - arity - 1,
                            };
                            self.frames.push(frame);
                            self.loc = CodeLocation::new(chunk.clone());
                            Ok(())
                        } else {
                            Err(Error::WrongArgCount {
                                expected: *arity,
                                found: argc,
                            })
                        }
                    }
                    Value::NativeFn { f, arity } => {
                        let begin = self.stack.len() - arity;
                        let result = f(&self.stack[begin..])?;
                        self.stack.drain(begin..);
                        self.stack.pop(); // Function object
                        self.stack.push(result);
                        Ok(())
                    }
                    _ => Err(Error::Value(value::Error::WrongCall(callable.clone()))),
                }
            }
            Instruction::Ret => {
                let frame = self.frames.pop().ok_or(Error::EmptyStack)?;
                self.loc = frame.call_loc;
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
            Instruction::Not => {
                let a = self.pop()?;
                self.stack.push(!a);
                Ok(())
            }
            Instruction::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a == b));
                Ok(())
            }
            Instruction::Less => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = if let Ordering::Less = a.cmp(&b)? {
                    true
                } else {
                    false
                };
                self.stack.push(Value::Bool(result));
                Ok(())
            }
            Instruction::Greater => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = if let Ordering::Greater = a.cmp(&b)? {
                    true
                } else {
                    false
                };
                self.stack.push(Value::Bool(result));
                Ok(())
            }
            Instruction::Temp => {
                panic!("Error during compilation: tried executing temporary instruction!")
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.loc.is_at_end() {
            if let e @ Err(_) = self.step() {
                return e;
            }
        }
        Ok(())
    }

    pub fn change_chunk(&mut self, chunk: Chunk) {
        self.loc = CodeLocation::new(chunk);
    }
}

pub type ValueError = value::Error;

#[derive(Debug)]
pub enum Error {
    Value(ValueError),
    Conversion(TryFromIntError),
    UndeclaredGlobal(String),
    WrongArgCount { expected: usize, found: u16 },
    EmptyStack,
    NoReturnValue,
}

impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Error::Value(err)
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Conversion(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Value(err) => write!(f, "{}", err),
            Error::Conversion(err) => write!(f, "Number too big to fit into VM code: {}", err),
            Error::UndeclaredGlobal(name) => write!(f, "Nonexistent variable '{}'", name),
            Error::WrongArgCount { expected, found } => write!(
                f,
                "Wrong argument count to function call: expected {}, found {}",
                expected, found
            ),
            Error::EmptyStack => write!(f, "Cannot return value out of an empty stack"),
            Error::NoReturnValue => write!(f, "Tried restoring value from empty return channel"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Value(err) => Some(err),
            Error::Conversion(err) => Some(err),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
