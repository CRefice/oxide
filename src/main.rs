mod parse;
mod scan;
mod vm;

use std::io::{self, BufRead as _};

use vm::Instruction;
use vm::Value;

#[derive(Debug)]
enum Error {
    Parse(parse::Error),
    VM(vm::Error),
}

impl From<parse::Error> for Error {
    fn from(e: parse::Error) -> Self {
        Error::Parse(e)
    }
}
impl From<vm::Error> for Error {
    fn from(e: vm::Error) -> Self {
        Error::VM(e)
    }
}

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let scanner = scan::Scanner::new(line.as_ref().unwrap());
        let parser = parse::Parser::new(scanner);
        let program = parser.parse()?;
        let mut vm = vm::VirtualMachine::new();
        vm.run(&program)?;
        println!("{:?}", vm.pop());
    }
    Ok(())
}
