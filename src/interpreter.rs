use crate::expr::Expression;
use crate::scan::Token;
use crate::stmt::Statement;
use crate::value::{Closure, Fn, Value, ValueError};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::ops;

#[derive(Clone)]
pub struct Scope<'a> {
    values: HashMap<String, Value<'a>>,
}

impl<'a> Scope<'a> {
    fn new() -> Self {
        Scope {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, val: Value<'a>) {
        self.values.insert(name, val);
    }

    fn get(&self, name: &str) -> Option<&Value<'a>> {
        self.values.get(name)
    }
}

#[derive(Debug)]
pub enum InterpretError<'a> {
    Value(ValueError<'a>),
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

impl<'a> From<ValueError<'a>> for InterpretError<'a> {
    fn from(e: ValueError<'a>) -> Self {
        InterpretError::Value(e)
    }
}

pub struct Interpreter<'a> {
    env: Vec<Scope<'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Interpreter<'a> {
        Interpreter {
            env: vec![Scope::new()],
        }
    }

    pub fn native_fn(&mut self, name: &str, f: &'a dyn ops::Fn(Vec<Value<'a>>) -> Value<'a>) {
        self.env
            .first_mut()
            .unwrap()
            .define(name.to_owned(), Value::Fn(Fn::Native(f)));
    }

    pub fn statement(&mut self, stmt: &Statement<'a>) -> Result<(), InterpretError<'a>> {
        match stmt {
            Statement::VarDecl { ident, init } => Ok({
                let val = self.evaluate(init)?;
                self.scope_mut().define(ident.identifier().to_owned(), val);
            }),
            Statement::FnDecl {
                ident,
                params,
                body,
            } => Ok({
                let f = Value::Fn(Fn::User {
                    closure: 1,
                    params: params.clone(),
                    body: body.clone(),
                });
                self.scope_mut().define(ident.identifier().to_owned(), f);
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

    pub fn evaluate(&mut self, ex: &Expression<'a>) -> Result<Value<'a>, InterpretError<'a>> {
        let cvterr = |e| InterpretError::from(e);
        match ex {
            Expression::Literal(x) => Ok(x.clone()),
            Expression::Variable(var) => self
                .get_var(var.identifier())
                .ok_or(InterpretError::VarNotFound(var.clone())),
            Expression::Call { callee, args } => match self.evaluate(callee)? {
                Value::Fn(f) => match f {
                    Fn::Native(f) => Ok(f(args
                        .into_iter()
                        .map(|e| self.evaluate(e).unwrap())
                        .collect())),
                    Fn::User {
                        closure,
                        params,
                        body,
                    } => {
                        if params.len() != args.len() {
                            panic!("Wrong number of arguments!")
                        }
                        let mut s = Scope::new();
                        for (param, arg) in params.into_iter().zip(args.iter()) {
                            if let Token::Identifier(name) = param {
                                s.define(name.into_owned(), self.evaluate(arg)?);
                            } else {
                                unreachable!()
                            }
                        }
                        self.env.push(s);
                        self.statement(&body)?;
                        self.env.pop();
                        Ok(Value::Bool(false))
                    }
                },
                f => Err(From::from(ValueError::WrongType(
                    f,
                    Value::Fn(Fn::Native(&(|_| Value::Bool(false)))),
                ))),
            },
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

    fn get_var(&self, name: &str) -> Option<Value<'a>> {
        self.get_from(self.env.len(), name)
    }

    fn get_from(&self, scope: usize, name: &str) -> Option<Value<'a>> {
        for scope in self.env[..scope].iter().rev() {
            if let x @ Some(_) = scope.get(name) {
                return x.map(|x| x.clone());
            }
        }
        None
    }

    fn assign(&mut self, ident: Token<'a>, val: Value<'a>) -> Result<(), InterpretError<'a>> {
        let name = ident.identifier();
        for scope in self.env.iter_mut().rev() {
            if scope.values.contains_key(name) {
                return Ok(scope.define(name.to_string(), val));
            }
        }
        Err(InterpretError::VarNotFound(ident))
    }

    fn scope(&self) -> &Scope<'a> {
        self.env.last().unwrap()
    }

    fn scope_mut(&mut self) -> &mut Scope<'a> {
        self.env.last_mut().unwrap()
    }
}
