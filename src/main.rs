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
    if let Some(file) = env::args().skip(1).next() {
        let contents = fs::read_to_string(&file).expect("Unable to open file");
        let mut interp = Interpreter::new();
        interp.load_libs();
        if let Err(e) = interp.run(&contents) {
            eprintln!("error:");
            if let Some(line) = e
                .location()
                .and_then(|(line, _)| contents.lines().nth(line - 1))
            {
                eprintln!("{}", line);
            }
            eprintln!("{}", e);
            std::process::exit(1);
        }
    } else {
        println!("Usage: reel [FILE]");
        std::process::exit(1);
    }
}
