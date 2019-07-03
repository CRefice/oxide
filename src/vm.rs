mod value;

pub use value::Value;

pub enum Instruction {
    Push(Value),
    GetLocal(u16),
    SetLocal(u16),
    Pop,
    PopFrame(usize),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
}

#[derive(Debug)]
pub enum Error {
    Value(value::Error),
    EmptyStack,
}

impl From<value::Error> for Error {
    fn from(err: value::Error) -> Self {
        Error::Value(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct VirtualMachine {
    stack: Vec<Value>,
    ip: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            stack: Vec::new(),
            ip: 0,
        }
    }

    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::EmptyStack)
    }

    pub fn peek(&mut self) -> Result<Value> {
        self.stack.last().cloned().ok_or(Error::EmptyStack)
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
            Instruction::GetLocal(idx) => {
                let idx = usize::from(*idx);
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
                let idx = usize::from(*idx);
                self.stack[idx] = val;
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
