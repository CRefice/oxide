#[macro_use]
mod value;
mod environment;
mod expr;
mod interpreter;
mod parse;
mod stmt;
mod token;

use std::env;
use std::fs;

use crate::interpreter::Interpreter;

fn main() {
    let mut interp = Interpreter::new();
    interp.load_libs();
    if let Some(file) = env::args().nth(1) {
        let contents = fs::read_to_string(&file).expect("Unable to open file");
        let mut interp = Interpreter::new();
        interp.load_libs();
        if let Err(e) = interp.run(&contents) {
            eprint!("error: ");
            if let Some((line, col)) = e.location() {
                eprintln!("{}: {}:{}:\n", &file, line, col);
                if let Some(line) = contents.lines().nth(line - 1) {
                    eprintln!("\t{}\n", line);
                }
            }
            eprintln!("{}", e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = interp.repl() {
            eprintln!("error: {}", e);
        }
    }
}
