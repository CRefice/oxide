use super::Scope;

use crate::expr::Expression;
use crate::scan::Token;
use crate::stmt::Statement;
use crate::value::{Fn, Value, ValueError};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub enum InterpretError<'a> {
    Value(ValueError<'a>),
    VarNotFound(Token<'a>),
    WrongArgCount { expected: usize, found: usize },
}

impl<'a> Display for InterpretError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InterpretError::Value(err) => write!(f, "{}", err),
            InterpretError::VarNotFound(name) => {
                write!(f, "Variable '{}' not found", name.identifier())
            }
            InterpretError::WrongArgCount { expected, found } => write!(
                f,
                "Wrong number of arguments supplied to function: found {}, expected {}",
                found, expected
            ),
        }
    }
}

impl<'a> From<ValueError<'a>> for InterpretError<'a> {
    fn from(e: ValueError<'a>) -> Self {
        InterpretError::Value(e)
    }
}

pub struct Interpreter<'a> {
    ret: Option<Value<'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Interpreter<'a> {
        Interpreter { ret: None }
    }

    pub fn statement(
        &mut self,
        stmt: &Statement<'a>,
        scope: Rc<RefCell<Scope<'a>>>,
    ) -> Result<(), InterpretError<'a>> {
        if self.ret.is_some() {
            return Ok(());
        };
        match stmt {
            Statement::VarDecl { ident, init } => Ok({
                let val = self.evaluate(init, scope.clone())?;
                scope.borrow_mut().define(ident.identifier(), val);
            }),
            Statement::FnDecl {
                ident,
                params,
                body,
            } => Ok({
                let f = Value::Fn(Fn::User {
                    closure: scope.clone(),
                    params: params.clone(),
                    body: body.clone(),
                });
                scope.borrow_mut().define(ident.identifier(), f);
            }),
            Statement::Expression(expr) => Ok({
                self.evaluate(expr, scope)?;
            }),
            Statement::If { cond, succ, fail } => Ok({
                let cond = self.evaluate(cond, scope.clone())?.is_truthy()?;
                if cond {
                    self.statement(succ, scope)?;
                } else if let Some(fail) = fail {
                    self.statement(fail, scope)?;
                }
            }),
            Statement::While { cond, stmt } => Ok({
                while self.evaluate(cond, scope.clone())?.is_truthy()? {
                    self.statement(stmt, scope.clone())?;
                }
            }),
            Statement::Return(expr) => Ok({
                self.ret = Some(
                    expr.as_ref()
                        .map(|e| self.evaluate(e, scope))
                        .unwrap_or(Ok(Value::Void))?,
                );
            }),
            Statement::Block(stmts) => Ok({
                let scope = Rc::new(RefCell::new(Scope::from(scope.clone())));
                for s in stmts {
                    self.statement(s, scope.clone())?;
                }
            }),
        }
    }

    pub fn evaluate(
        &mut self,
        ex: &Expression<'a>,
        scope: Rc<RefCell<Scope<'a>>>,
    ) -> Result<Value<'a>, InterpretError<'a>> {
        let cvterr = |e| InterpretError::from(e);
        match ex {
            Expression::Literal(x) => Ok(x.clone()),
            Expression::Variable(var) => scope
                .borrow()
                .get(var.identifier())
                .ok_or(InterpretError::VarNotFound(var.clone())),
            Expression::Call { callee, args } => match self.evaluate(callee, scope.clone())? {
                Value::Fn(f) => match f {
                    Fn::Native { arity, f } => {
                        if args.len() != arity {
                            Err(InterpretError::WrongArgCount {
                                expected: arity,
                                found: args.len(),
                            })
                        } else {
                            let mut v = Vec::new();
                            for arg in args.into_iter() {
                                v.push(self.evaluate(arg, scope.clone())?);
                            }
                            Ok(f(v))
                        }
                    }
                    Fn::User {
                        closure,
                        params,
                        body,
                    } => {
                        if params.len() != args.len() {
                            return Err(InterpretError::WrongArgCount {
                                expected: params.len(),
                                found: args.len(),
                            });
                        }
                        let mut s = Scope::from(closure);
                        for (param, arg) in params.into_iter().zip(args.iter()) {
                            s.define(param.identifier(), self.evaluate(arg, scope.clone())?);
                        }
                        self.statement(&body, Rc::new(RefCell::new(s)))?;
                        let ret = self.ret.take().as_ref().cloned().unwrap_or(Value::Void);
                        Ok(ret)
                    }
                },
                f => Err(From::from(ValueError::WrongType(
                    f,
                    Value::Fn(Fn::Native {
                        arity: 0,
                        f: &(|_| Value::Bool(false)),
                    }),
                ))),
            },
            Expression::Assignment { ident, val } => {
                let val = self.evaluate(val, scope.clone())?;
                scope
                    .clone()
                    .borrow_mut()
                    .assign(ident.identifier(), val)
                    .ok_or(InterpretError::VarNotFound(ident.clone()))
            }
            Expression::Grouping(b) => self.evaluate(b, scope),
            Expression::Unary(op, right) => {
                let val = self.evaluate(right, scope)?;
                match op {
                    Token::Minus => (-val).map_err(cvterr),
                    Token::Bang => (!val).map_err(cvterr),
                    _ => panic!("Unrecognized unary operator"),
                }
            }
            Expression::Logical(left, op, right) => {
                let val = (self.evaluate(left, scope.clone())?).is_truthy()?;
                match op {
                    Token::And => Ok(Value::Bool(
                        val && self.evaluate(right, scope)?.is_truthy()?,
                    )),
                    Token::Or => Ok(Value::Bool(
                        val || self.evaluate(right, scope)?.is_truthy()?,
                    )),
                    _ => panic!("Unrecognized logical operator"),
                }
            }
            Expression::Binary(left, op, right) => {
                let left = self.evaluate(left, scope.clone())?;
                let right = self.evaluate(right, scope.clone())?;
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
}
