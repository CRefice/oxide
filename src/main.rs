mod expr;
mod interpreter;
mod parse;
mod stmt;
mod scan;
#[macro_use]
mod value;

use crate::interpreter::Interpreter;
use std::env;
use std::io;
use std::fs;
use std::cell::RefCell;
use std::rc::Rc;

fn loc_from_index(i: usize, s: &str) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for c in s.chars().take(i) {
        if c == '\n' {
            line = line + 1;
            col = 0;
        } else {
            col = col + 1;
        }
    }
    (line, col)
}

fn from_file(file: &str) {
    let contents = fs::read_to_string(file).expect("Unable to read file");
    let mut intrp = Interpreter::new();
    let mut scope = interpreter::Scope::new();
    scope.define("print", function!(a, {
        println!("{}", a);
        value::Value::Void
    }));
    scope.define("read", function!(, {
        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();
        match s.trim().parse() {
            Ok(x) => value::Value::Num(x),
            Err(e) => {
                println!("Parse error: {}", e);
                std::process::exit(1);
            }
        }
    }));
    let lexer = scan::Lexer::new(&contents);
    let mut parser = parse::Parser::new(lexer);
    let handle = Rc::new(RefCell::new(scope));
    match parser.program() {
        Ok(stmts) => for s in stmts.iter() {
            if let Err(e) = intrp.statement(&s, handle.clone()) {
                println!("Interpret error: {}", e);
            }
        }
        Err(e) => {
            let (line, col) = loc_from_index(e.location(), &contents);
            println!("{}: Syntax error", file);
            println!("{}:{}: {}", line, col, e);
        }
    }
}

fn main() {
    if let Some(file) = env::args().skip(1).next() {
        from_file(&file);
    } else {
        println!("Usage: reel [FILE]");
        std::process::exit(1);
    }
}
