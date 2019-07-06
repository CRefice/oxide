mod libs;

use std::fs::File;
use std::io::{self, BufRead as _, Read as _};

use crate::compile::{self, Compiler};
use crate::scan::Scanner;
use crate::vm::{self, VirtualMachine};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Compilation(compile::Error),
    VM(vm::Error),
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
        Error::VM(e)
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

    pub fn run_file(&mut self, mut file: File) -> Result<()> {
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        let mut compiler = Compiler::new();
        let mut scanner = Scanner::new(&text).peekable();
        compiler.program(&mut scanner)?;
        self.vm.run(compiler.instructions())?;
        Ok(())
    }

    pub fn repl(&mut self) -> Result<()> {
        let mut compiler = Compiler::new();
        for line in io::stdin().lock().lines() {
            let mut scanner = Scanner::new(line.as_ref().unwrap()).peekable();
            compiler.declaration(&mut scanner)?;
            self.vm.run(compiler.instructions())?;
            println!("{:?}", self.vm.pop()?);
        }
        Ok(())
    }
}
