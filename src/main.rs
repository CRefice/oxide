mod token;
mod expr;
mod value;
mod parse;
mod context;

use std::io;

fn main() {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let lexer = token::Lexer::new(&input);
        let mut context = context::Context::new();
        let mut parser = parse::Parser::new(lexer);
        let expr = parser.expression();
        match context.evaluate(expr) {
            Ok(val) => println!("{}", val),
            Err(err) => println!("{}", err)
        }
    }
}
