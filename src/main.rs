mod interpreter;
mod expr;
mod parse;
mod stmt;
mod token;
mod value;

use std::io;
use crate::interpreter::Interpreter;

fn main() {
    let mut interp = Interpreter::new();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let lexer = token::Lexer::new(&input);
        let mut parser = parse::Parser::new(lexer.clone());
        match parser.declaration() {
            Ok(stmt) => {
                match interp.statement(stmt) {
                    Ok(_) => (),
                    Err(err) => println!("{}", err),
                }
            }
            Err(err) => {
                let mut parser = parse::Parser::new(lexer);
                match parser.expression() {
                    Ok(expr) => {
                        match interp.evaluate(expr) {
                            Ok(val) => println!("{}", val),
                            Err(err) => println!("{}", err),
                        }
                    }
                    Err(err2) => {
                        println!("Error parsing statement: {}", err);
                        println!("Error parsing expression: {}", err2);
                    }
                }
            }
        }
    }
}
