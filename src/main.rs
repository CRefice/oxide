mod expr;
mod interpreter;
mod parse;
mod stmt;
mod token;
mod value;

use crate::interpreter::Interpreter;
use std::env;
use std::fs;
use std::io;

fn repl() {
    let mut interp = Interpreter::new();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let lexer = token::Lexer::new(&input);
        let mut parser = parse::Parser::new(lexer.clone());
        match parser.declaration() {
            Ok(stmt::Statement::Expression(expr)) => match interp.evaluate(expr) {
                Ok(val) => println!("{}", val),
                Err(err) => println!("Evaluation error: {}", err),
            },
            Ok(stmt) => if let Err(err) = interp.statement(stmt) {
                println!("Interpret error: {}", err)
            }
            Err(err) => {
                println!("Parse error: {}", err);
            }
        }
    }
}

fn from_file(file: &str) {
    let contents = fs::read_to_string(file).expect("Unable to read file");
    let mut intrp = Interpreter::new();
    let lexer = token::Lexer::new(&contents);
    let mut parser = parse::Parser::new(lexer);
    while !parser.done() {
        if let Err(e) = parser
            .declaration()
            .map_err(|e| format!("{}", e))
            .and_then(|stmt| intrp.statement(stmt).map_err(|e| format!("{}", e)))
        {
            println!("{}", e);
            break;
        }
    }
    intrp.print_state();
}

fn main() {
    if let Some(file) = env::args().skip(1).next() {
        from_file(&file);
    } else {
        repl();
    }
}
