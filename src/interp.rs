mod libs;

use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, Read as _};
use std::path::Path;
use std::rc::Rc;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::compile::{self, Compiler};
use crate::loc::{SourceLocation, TryLocate};
use crate::scan::TokenStream;
use crate::vm::{self, Value, VirtualMachine};

pub fn run_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut text = String::new();
    let mut file = File::open(path.as_ref())?;
    file.read_to_string(&mut text)?;
    let mut compiler = Compiler::new();
    let mut stream = TokenStream::new(&text).peekable();
    compiler.program(&mut stream)?;
    let chunk = compiler.instructions();
    let mut vm = VirtualMachine::new(Rc::new(chunk));
    libs::load_libraries(&mut vm);
    vm.run()?;
    Ok(())
}

pub fn repl() {
    let mut rl = Editor::<()>::new();
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new(Rc::new(Vec::new()));
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let line = line.as_str();
                rl.add_history_entry(line);
                match run_line(line, &mut compiler, &mut vm) {
                    Ok(val) => println!("{}", val),
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn run_line(text: &str, compiler: &mut Compiler, vm: &mut VirtualMachine) -> Result<Value> {
    let mut stream = TokenStream::new(text).peekable();
    compiler.declaration(&mut stream)?;
    let chunk = Rc::new(compiler.instructions());
    vm.change_chunk(chunk);
    vm.run()?;
    Ok(vm.pop()?)
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Compilation(compile::Error),
    Runtime(vm::Error),
}

impl TryLocate for Error {
    fn maybe_location(&self) -> Option<SourceLocation> {
        match self {
            Error::IO(err) => None,
            Error::Compilation(err) => err.maybe_location(),
            Error::Runtime(err) => None,
        }
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

type Result<T> = std::result::Result<T, Error>;
