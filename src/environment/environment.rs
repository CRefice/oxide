use super::Error;
use super::{Scope, ScopeHandle};

use crate::expr::Expression;
use crate::stmt::Statement;
use crate::token;
use crate::value::{self, Fn, Value};
use std::cmp::Ordering;

pub struct Environment {
    ret: Option<Value>,
}

pub type Result<T> = std::result::Result<T, super::Error>;

impl Environment {
    pub fn new() -> Environment {
        Environment { ret: None }
    }

    pub fn statement(&mut self, stmt: &Statement, scope: ScopeHandle) -> Result<()> {
        if self.ret.is_some() {
            return Ok(());
        };
        match stmt {
            Statement::VarDecl { ident, init } => {
                let val = self.evaluate(init, scope.clone())?;
                scope.borrow_mut().define(ident.identifier(), val);
                Ok(())
            }
            Statement::FnDecl {
                ident,
                params,
                body,
            } => {
                let f = Value::Fn(Fn::User {
                    closure: scope.clone(),
                    params: params.clone(),
                    body: body.clone(),
                });
                scope.borrow_mut().define(ident.identifier(), f);
                Ok(())
            }
            Statement::Expression(expr) => {
                self.evaluate(expr, scope)?;
                Ok(())
            }
            Statement::If {
                cond,
                succ,
                fail,
                loc,
            } => {
                let cond = self
                    .evaluate(cond, scope.clone())?
                    .is_truthy()
                    .map_err(|err| Error::Value { err, loc: *loc })?;
                if cond {
                    self.statement(succ, scope)?;
                } else if let Some(fail) = fail {
                    self.statement(fail, scope)?;
                }
                Ok(())
            }
            Statement::While { cond, stmt, loc } => {
                while self
                    .evaluate(cond, scope.clone())?
                    .is_truthy()
                    .map_err(|err| Error::Value { err, loc: *loc })?
                {
                    self.statement(stmt, scope.clone())?;
                }
                Ok(())
            }
            Statement::Return(expr) => {
                self.ret = Some(
                    expr.as_ref()
                        .map(|e| self.evaluate(e, scope))
                        .unwrap_or(Ok(Value::Void))?,
                );
                Ok(())
            }
            Statement::Block(stmts) => {
                let scope = ScopeHandle::from(Scope::from(scope.clone()));
                for s in stmts {
                    self.statement(s, scope.clone())?;
                }
                Ok(())
            }
        }
    }

    pub fn evaluate(
        &mut self,
        ex: &Expression,
        scope: ScopeHandle,
    ) -> Result<Value> {
        match ex {
            Expression::Literal(x) => Ok(x.clone()),
            Expression::Variable(var) => scope
                .borrow()
                .get(var.identifier())
                .ok_or_else(|| Error::VarNotFound(var.clone())),
            Expression::Call { callee, args, loc } => match self.evaluate(callee, scope.clone())? {
                Value::Fn(f) => match f {
                    Fn::Native { arity, f } => {
                        if args.len() != arity {
                            Err(Error::WrongArgCount {
                                expected: arity,
                                found: args.len(),
                                loc: *loc,
                            })
                        } else {
                            let mut v = Vec::new();
                            for arg in args.iter() {
                                v.push(self.evaluate(arg, scope.clone())?);
                            }
                            f(v).map_err(|err| Error::Value { err, loc: *loc })
                        }
                    }
                    Fn::User {
                        closure,
                        params,
                        body,
                    } => {
                        if params.len() != args.len() {
                            return Err(Error::WrongArgCount {
                                expected: params.len(),
                                found: args.len(),
                                loc: *loc,
                            });
                        }
                        let mut s = Scope::from(closure);
                        for (param, arg) in params.into_iter().zip(args.iter()) {
                            s.define(param.identifier(), self.evaluate(arg, scope.clone())?);
                        }
                        self.statement(&body, ScopeHandle::from(s))?;
                        let ret = self.ret.take().as_ref().cloned().unwrap_or(Value::Void);
                        Ok(ret)
                    }
                },
                f => Err(Error::Value {
                    err: value::Error::WrongType(f, function!(, Ok(Value::Void))),
                    loc: *loc,
                }),
            },
            Expression::Indexing {
                operand,
                index,
                loc,
            } => {
                let operand = self.evaluate(operand, scope.clone())?;
                let index = self.evaluate(index, scope.clone())?;
                operand
                    .index(&index)
                    .map_err(|err| Error::Value { err, loc: *loc })
            }
            Expression::Assignment { ident, val } => {
                let val = self.evaluate(val, scope.clone())?;
                scope
                    .borrow_mut()
                    .assign(ident.identifier(), val)
                    .ok_or_else(|| Error::VarNotFound(ident.clone()))
            }
            Expression::IndexingAssignment {
                ident,
                index,
                val,
                loc,
            } => {
                let index = self.evaluate(index, scope.clone())?;
                let val = self.evaluate(val, scope.clone())?;
                scope
                    .borrow_mut()
                    .assign_index(ident.identifier(), index, val)
                    .ok_or_else(|| Error::VarNotFound(ident.clone()))?
                    .map_err(|err| Error::Value { err, loc: *loc })
            }
            Expression::Grouping(b) => self.evaluate(b, scope),
            Expression::Array(arr) => {
                let mut vec = Vec::new();
                for expr in arr {
                    vec.push(self.evaluate(expr, scope.clone())?);
                }
                Ok(Value::Array(vec))
            }
            Expression::Unary { op, operand } => {
                let val = self.evaluate(operand, scope)?;
                match op.kind {
                    token::Minus => (-val).map_err(|err| Error::Value { err, loc: op.loc }),
                    token::Bang => (!val).map_err(|err| Error::Value { err, loc: op.loc }),
                    _ => panic!("Unrecognized unary operator"),
                }
            }
            Expression::Logical { a, b, op } => {
                let val = self
                    .evaluate(a, scope.clone())?
                    .is_truthy()
                    .map_err(|err| Error::Value { err, loc: op.loc })?;
                match op.kind {
                    token::And => Ok(Value::Bool(
                        val && self
                            .evaluate(b, scope)?
                            .is_truthy()
                            .map_err(|err| Error::Value { err, loc: op.loc })?,
                    )),
                    token::Or => Ok(Value::Bool(
                        val || self
                            .evaluate(b, scope)?
                            .is_truthy()
                            .map_err(|err| Error::Value { err, loc: op.loc })?,
                    )),
                    _ => panic!("Unrecognized logical operator"),
                }
            }
            Expression::Binary { a, b, op } => {
                let left = self.evaluate(a, scope.clone())?;
                let right = self.evaluate(b, scope.clone())?;
                match op.kind {
                    token::Plus => (left + right).map_err(|err| Error::Value { err, loc: op.loc }),
                    token::Minus => (left - right).map_err(|err| Error::Value { err, loc: op.loc }),
                    token::Star => (left * right).map_err(|err| Error::Value { err, loc: op.loc }),
                    token::Slash => (left / right).map_err(|err| Error::Value { err, loc: op.loc }),
                    token::EqualEqual => Ok(Value::Bool(left == right)),
                    token::BangEqual => Ok(Value::Bool(left != right)),
                    token::Greater => {
                        let b = if let Ordering::Greater = left
                            .compare(&right)
                            .map_err(|err| Error::Value { err, loc: op.loc })?
                        {
                            true
                        } else {
                            false
                        };
                        Ok(Value::Bool(b))
                    }
                    token::GreaterEqual => {
                        let b = match left
                            .compare(&right)
                            .map_err(|err| Error::Value { err, loc: op.loc })?
                        {
                            Ordering::Greater | Ordering::Equal => true,
                            _ => false,
                        };
                        Ok(Value::Bool(b))
                    }
                    token::Less => {
                        let b = if let Ordering::Less = left
                            .compare(&right)
                            .map_err(|err| Error::Value { err, loc: op.loc })?
                        {
                            true
                        } else {
                            false
                        };
                        Ok(Value::Bool(b))
                    }
                    token::LessEqual => {
                        let b = match left
                            .compare(&right)
                            .map_err(|err| Error::Value { err, loc: op.loc })?
                        {
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
