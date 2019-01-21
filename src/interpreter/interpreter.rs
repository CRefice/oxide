use std::io;

use crate::environment::{Environment, Scope, ScopeHandle};
use crate::parse::Parser;
use crate::token::Lexer;
use crate::stmt::Statement;

pub struct Interpreter {
    env: Environment,
    globals: ScopeHandle,
}

type Result<T> = std::result::Result<T, super::Error>;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
            globals: ScopeHandle::from(Scope::new()),
        }
    }

    pub fn run(&mut self, contents: &str) -> Result<()> {
        let mut parser = Parser::new(Lexer::new(contents));
        let scope = ScopeHandle::from(Scope::from(self.globals.clone()));
        let stmts = parser.program()?;
        for s in stmts.iter() {
            self.env.statement(s, scope.clone())?;
        }
        Ok(())
    }

    pub fn repl(&mut self) -> Result<()> {
        let scope = ScopeHandle::from(Scope::from(self.globals.clone()));
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let mut parser = Parser::new(Lexer::new(&line));
            for stmt in parser.program()? {
                if let Statement::Expression(expr) = stmt {
                    println!("{}", self.env.evaluate(&expr, scope.clone())?);
                } else {
                    self.env.statement(&stmt, scope.clone())?;
                }
            }
        }
    }

    pub fn load_libs(&mut self) {
        super::libs::load_libs(&mut self.globals.borrow_mut());
    }
}
