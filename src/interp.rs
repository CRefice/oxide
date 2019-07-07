mod libs;

use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufRead as _, Read as _};
use std::path::Path;

use crate::compile::{self, Compiler};
use crate::scan::Scanner;
use crate::vm::{self, Value, VirtualMachine};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Compilation(compile::Error),
    Runtime(vm::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<compile::Error> for Error {
    fn from(e: compile::Error) -> Self {
        Error::Compilation(e)
    }
}

impl From<vm::Error> for Error {
    fn from(e: vm::Error) -> Self {
        Error::Runtime(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "{}", err),
            Error::Compilation(err) => write!(f, "Compilation error: {}", err),
            Error::Runtime(err) => write!(f, "Runtime error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            Error::Compilation(err) => Some(err),
            Error::Runtime(err) => Some(err),
        }
    }
}

pub struct Interpreter {
    vm: VirtualMachine,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut vm = VirtualMachine::new();
        libs::load_libraries(&mut vm);
        Interpreter { vm }
    }

    pub fn run_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut text = String::new();
        let mut file = File::open(path.as_ref())?;
        file.read_to_string(&mut text)?;
        let mut compiler = Compiler::new();
        let mut scanner = Scanner::new(&text).peekable();
        compiler.program(&mut scanner)?;
        self.vm.run(compiler.instructions())?;
        Ok(())
    }

    pub fn repl(&mut self) {
        let mut compiler = Compiler::new();
        for line in io::stdin().lock().lines() {
            match self.run_line(line.as_ref().unwrap(), &mut compiler) {
                Ok(val) => println!("{}", val),
                Err(err) => println!("{}", err),
            }
        }
    }

    fn run_line(&mut self, text: &str, compiler: &mut Compiler) -> Result<Value> {
        let mut scanner = Scanner::new(text).peekable();
        compiler.declaration(&mut scanner)?;
        self.vm.run(compiler.instructions())?;
        Ok(self.vm.pop()?)
    }
}