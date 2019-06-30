mod value;

pub use value::Value;

pub struct VirtualMachine {
    stack: Vec<Value>,
    ip: usize,
}

pub enum Instruction {
    Push(Value),
    Pop,
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

    pub fn step(&mut self, code: &[Instruction]) -> Result<()> {
        let result = match &code[self.ip] {
            Instruction::Push(val) => {
                self.stack.push(val.clone());
                Ok(())
            }
            Instruction::Pop => self.pop().map(|_| ()),
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
