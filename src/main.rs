mod token;
mod expr;
mod value;

use std::io;

fn main() {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let lexer = token::Lexer::new(&input);
        let mut parser = expr::Parser::new(lexer);
        let expr = parser.parse();
        match expr::evaluate(expr) {
            Ok(val) => println!("{}", val),
            Err(err) => println!("{}", err)
        }
    }
}
