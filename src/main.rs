mod context;
mod expr;
mod parse;
mod stmt;
mod token;
mod value;

use std::io;

fn main() {
    let mut context = context::Interpreter::new();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let lexer = token::Lexer::new(&input);
        let mut parser = parse::Parser::new(lexer.clone());
        match parser.declaration() {
            Ok(stmt) => {
                match context.statement(stmt) {
                    Ok(_) => (),
                    Err(err) => println!("{}", err),
                }
            }
            Err(err) => {
                let mut parser = parse::Parser::new(lexer);
                match parser.expression() {
                    Ok(expr) => {
                        match context.evaluate(expr) {
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
