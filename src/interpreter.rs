use crate::expr::Expression;
use crate::stmt::Statement;
use crate::token::Token;
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

    fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.values.get_mut(name)
    }
}

#[derive(Debug)]
pub enum InterpretError<'a> {
    Value(ValueError),
    VarNotFound(&'a str),
}

impl<'a> Display for InterpretError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InterpretError::Value(err) => write!(f, "{}", err),
            InterpretError::VarNotFound(name) => write!(f, "Variable '{}' not found", name),
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
        Interpreter { env: vec![Scope::new()] }
    }

    pub fn statement<'a>(&mut self, stmt: Statement<'a>) -> Result<(), InterpretError<'a>> {
        match stmt {
            Statement::VarDecl{name, init} => Ok({
                let val = self.evaluate(init)?;
                self.scope_mut().define(name.to_string(), val);
            }),
            Statement::Expression(expr) => Ok({
                self.evaluate(expr)?;
            }),
            Statement::If{cond, succ, fail} => {
                match self.evaluate(cond)? {
                    Value::Bool(x) => Ok({
                        if x {
                            self.statement(*succ)?;
                        } else if let Some(fail) = fail {
                            self.statement(*fail)?;
                        }
                    }),
                    x => Err(From::from(ValueError::WrongType(Value::Bool(false), x)))
                }
            },
            Statement::Block(stmts) => Ok({
                self.env.push(Scope::new());
                for s in stmts {
                    self.statement(s)?;
                }
                self.env.pop();
            }),
        }
    }

    pub fn evaluate<'a>(&self, ex: Expression<'a>) -> Result<Value, InterpretError<'a>> {
        let cvterr = |e| InterpretError::from(e);
        match ex {
            Expression::Literal(x) => Ok(x),
            Expression::Variable(var) => self
                .get_var(var)
                .ok_or(InterpretError::VarNotFound(var)),
            Expression::Grouping(b) => self.evaluate(*b),
            Expression::Unary(op, right) => {
                let val = self.evaluate(*right)?;
                match op {
                    Token::Minus => (-val).map_err(cvterr),
                    Token::Bang => (!val).map_err(cvterr),
                    _ => panic!("Unrecognized unary operator"),
                }
            }
            Expression::Binary(left, op, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;
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

    fn get_var(&self, name: &str) -> Option<Value> {
        for scope in self.env.iter().rev() {
            if let x@Some(_) = scope.get(name) {
                return x.map(|x| x.clone());
            }
        }
        None
    }

    fn scope(&self) -> &Scope {
        self.env.last().unwrap()
    }

    pub fn print_state(&self) {
        for (k, v) in self.scope().values.iter() {
            println!("{}: {}", k, v);
        }
    }

    fn scope_mut(&mut self) -> &mut Scope {
        self.env.last_mut().unwrap()
    }
}
