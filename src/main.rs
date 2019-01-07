mod expr;
mod interpreter;
mod parse;
mod stmt;
mod scan;
mod value;

use crate::interpreter::Interpreter;
use std::env;
use std::fs;
use std::io;

fn repl() {
    //let mut interp = Interpreter::new();
    //loop {
    //    let mut input = String::new();
    //    io::stdin().read_line(&mut input).unwrap();
    //    let lexer = scan::Lexer::new(&input);
    //    let mut parser = parse::Parser::new(lexer.map(|(l, t)| (l, t.to_owned())));
    //    match parser.declaration() {
    //        Ok(stmt::Statement::Expression(expr)) => match interp.evaluate(&expr) {
    //            Ok(val) => println!("{}", val),
    //            Err(err) => println!("Evaluation error: {}", err),
    //        },
    //        Ok(stmt) => if let Err(err) = interp.statement(&stmt) {
    //            println!("Interpret error: {}", err)
    //        }
    //        Err(err) => {
    //            println!("Parse error: {}", err);
    //        }
    //    }
    //}
}

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
    let f = |vec: Vec<value::Value>| {
        println!("{}", vec.first().unwrap());
        value::Value::Bool(false)
    };
    intrp.native_fn("print", &(f));
    let lexer = scan::Lexer::new(&contents);
    let mut parser = parse::Parser::new(lexer);
    match parser.program() {
        Ok(stmts) => for s in stmts.iter() {
            if let Err(e) = intrp.statement(&s) {
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
        repl();
    }
}
