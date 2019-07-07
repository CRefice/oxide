mod value;

use std::collections::HashMap;
use std::convert::TryInto as _;
use std::fmt::{self, Display};
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
    Not,
    Equal,
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
        let opcode = &code[self.ip];
        self.ip += 1;
        match opcode {
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
                let cond = self.peek()?;
                if !cond.is_truthy() {
                    self.jump(*offset)?;
                }
                Ok(())
            }
            Instruction::Call(argc) => {
                let arg_len = usize::from(*argc);
                // TODO: Remove this once/if you implement an AST.
                let args = self
                    .stack
                    .drain(self.stack.len() - arg_len..)
                    .collect::<Vec<_>>();
                let callable = self.pop()?;
                self.stack.extend(args);
                match callable {
                    Value::Function { code_loc, arity } => {
                        if arg_len == arity {
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
        }
    }

    pub fn run(&mut self, code: &[Instruction]) -> Result<()> {
        while self.ip < code.len() {
            if let e @ Err(_) = self.step(code) {
                self.ip = code.len();
                return e;
            }
        }
        Ok(())
    }
}
