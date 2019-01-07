use crate::expr::Expression;
use crate::scan::Token;
use crate::stmt::Statement;
use crate::value::{Value, ValueError};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

struct Scope {
    values: HashMap<String, Value>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, val: Value) {
        self.values.insert(name, val);
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }
}

#[derive(Debug)]
pub enum InterpretError<'a> {
    Value(ValueError),
    VarNotFound(Token<'a>),
}

impl<'a> Display for InterpretError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InterpretError::Value(err) => write!(f, "{}", err),
            InterpretError::VarNotFound(name) => write!(f, "Variable '{:?}' not found", name),
        }
    }
}

impl<'a> From<ValueError> for InterpretError<'a> {
    fn from(e: ValueError) -> Self {
        InterpretError::Value(e)
    }
}

pub struct Interpreter {
    env: Vec<Scope>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            env: vec![Scope::new()],
        }
    }

    pub fn statement<'a>(&mut self, stmt: &'a Statement<'a>) -> Result<(), InterpretError<'a>> {
        match stmt {
            Statement::VarDecl { ident, init } => Ok({
                let val = self.evaluate(init)?;
                self.scope_mut().define(ident.identifier().to_owned(), val);
            }),
            Statement::Expression(expr) => Ok({
                self.evaluate(expr)?;
            }),
            Statement::If { cond, succ, fail } => Ok({
                let cond = self.evaluate(cond)?.as_bool()?;
                if cond {
                    self.statement(succ)?;
                } else if let Some(fail) = fail {
                    self.statement(fail)?;
                }
            }),
            Statement::While { cond, stmt } => Ok({
                while self.evaluate(cond)?.as_bool()? {
                    self.statement(stmt)?;
                }
            }),
            Statement::Block(stmts) => Ok({
                self.env.push(Scope::new());
                for s in stmts {
                    self.statement(s)?;
                }
                self.env.pop();
            }),
        }
    }

    pub fn evaluate<'a>(&mut self, ex: &'a Expression<'a>) -> Result<Value, InterpretError<'a>> {
        let cvterr = |e| InterpretError::from(e);
        match ex {
            Expression::Literal(x) => Ok(x.clone()),
            Expression::Variable(var) => self
                .get_var(var.identifier())
                .ok_or(InterpretError::VarNotFound(var.clone())),
            Expression::Assignment { ident, val } => {
                let val = self.evaluate(val)?;
                self.assign(ident.clone(), val).and_then(|_| {
                    self.get_var(ident.identifier())
                        .ok_or(InterpretError::VarNotFound(ident.clone()))
                })
            }
            Expression::Grouping(b) => self.evaluate(b),
            Expression::Unary(op, right) => {
                let val = self.evaluate(right)?;
                match op {
                    Token::Minus => (-val).map_err(cvterr),
                    Token::Bang => (!val).map_err(cvterr),
                    _ => panic!("Unrecognized unary operator"),
                }
            }
            Expression::Logical(left, op, right) => {
                let val = (self.evaluate(left)?).as_bool()?;
                match op {
                    Token::And => Ok(Value::Bool(val && self.evaluate(right)?.as_bool()?)),
                    Token::Or => Ok(Value::Bool(val || self.evaluate(right)?.as_bool()?)),
                    _ => panic!("Unrecognized logical operator"),
                }
            }
            Expression::Binary(left, op, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                match op {
                    Token::Plus => (left + right).map_err(cvterr),
                    Token::Minus => (left - right).map_err(cvterr),
                    Token::Star => (left * right).map_err(cvterr),
                    Token::Slash => (left / right).map_err(cvterr),
                    Token::EqualEqual => left.equals(right).map_err(cvterr),
                    Token::BangEqual => left.equals(right).and_then(|c| !c).map_err(cvterr),
                    Token::Greater => {
                        let b = if let Ordering::Greater = left.compare(right)? {
                            true
                        } else {
                            false
                        };
                        Ok(Value::Bool(b))
                    }
                    Token::GreaterEqual => {
                        let b = match left.compare(right)? {
                            Ordering::Greater | Ordering::Equal => true,
                            _ => false,
                        };
                        Ok(Value::Bool(b))
                    }
                    Token::Less => {
                        let b = if let Ordering::Less = left.compare(right)? {
                            true
                        } else {
                            false
                        };
                        Ok(Value::Bool(b))
                    }
                    Token::LessEqual => {
                        let b = match left.compare(right)? {
                            Ordering::Less | Ordering::Equal => true,
                            _ => false,
                        };
                        Ok(Value::Bool(b))
                    }
                    _ => panic!("Unrecognized binary operator"),
                }
            }
        }
    }

    pub fn print_state(&self) {
        for (k, v) in self.scope().values.iter() {
            println!("{}: {}", k, v);
        }
    }

    fn get_var(&self, name: &str) -> Option<Value> {
        for scope in self.env.iter().rev() {
            if let x @ Some(_) = scope.get(name) {
                return x.map(|x| x.clone());
            }
        }
        None
    }

    fn assign<'a>(&mut self, ident: Token<'a>, val: Value) -> Result<(), InterpretError<'a>> {
        let name = ident.identifier();
        for scope in self.env.iter_mut().rev() {
            if scope.values.contains_key(name) {
                return Ok(scope.define(name.to_string(), val));
            }
        }
        Err(InterpretError::VarNotFound(ident))
    }

    fn scope(&self) -> &Scope {
        self.env.last().unwrap()
    }

    fn scope_mut(&mut self) -> &mut Scope {
        self.env.last_mut().unwrap()
    }
}
