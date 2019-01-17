use crate::environment::{Environment, Scope, ScopeHandle};
use crate::parse::Parser;
use crate::token::Lexer;

pub struct Interpreter<'a> {
    env: Environment<'a>,
    globals: ScopeHandle<'a>,
}

type Result<'a, T> = std::result::Result<T, super::Error<'a>>;

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
            globals: Scope::new().to_handle(),
        }
    }

    pub fn run(&mut self, contents: &'a str) -> Result<'a, ()> {
        let mut parser = Parser::new(Lexer::new(contents));
        let scope = Scope::from(self.globals.clone()).to_handle();
        let stmts = parser.program()?;
        for s in stmts.iter() {
            self.env.statement(s, scope.clone())?;
        }
        Ok(())
    }

    pub fn load_libs(&mut self) {
        super::libs::load_libs(&mut self.globals.borrow_mut());
    }
}
