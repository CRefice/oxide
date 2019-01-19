use super::Error;
use super::{Scope, ScopeHandle};

use crate::expr::Expression;
use crate::stmt::Statement;
use crate::token;
use crate::value::{self, Fn, Value};
use std::cmp::Ordering;

pub struct Environment<'a> {
    ret: Option<Value<'a>>,
}

type Result<'a, T> = std::result::Result<T, super::Error<'a>>;

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment { ret: None }
    }

    pub fn statement(&mut self, stmt: &Statement<'a>, scope: ScopeHandle<'a>) -> Result<'a, ()> {
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
            Statement::If {
                cond,
                succ,
                fail,
                loc,
            } => Ok({
                let cond = self
                    .evaluate(cond, scope.clone())?
                    .is_truthy()
                    .map_err(|err| Error::Value { err, loc: *loc })?;
                if cond {
                    self.statement(succ, scope)?;
                } else if let Some(fail) = fail {
                    self.statement(fail, scope)?;
                }
            }),
            Statement::While { cond, stmt, loc } => Ok({
                while self
                    .evaluate(cond, scope.clone())?
                    .is_truthy()
                    .map_err(|err| Error::Value { err, loc: *loc })?
                {
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
                let scope = Scope::from(scope.clone()).to_handle();
                for s in stmts {
                    self.statement(s, scope.clone())?;
                }
            }),
        }
    }

    pub fn evaluate(
        &mut self,
        ex: &Expression<'a>,
        scope: ScopeHandle<'a>,
    ) -> Result<'a, Value<'a>> {
        match ex {
            Expression::Literal(x) => Ok(x.clone()),
            Expression::Variable(var) => scope
                .borrow()
                .get(var.identifier())
                .ok_or(Error::VarNotFound(var.clone())),
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
                        self.statement(&body, s.to_handle())?;
                        let ret = self.ret.take().as_ref().cloned().unwrap_or(Value::Void);
                        Ok(ret)
                    }
                },
                f => Err(Error::Value {
                    err: value::Error::WrongType(
                        f,
                        Value::Fn(Fn::Native {
                            arity: 0,
                            f: &(|_| Value::Bool(false)),
                        }),
                    ),
                    loc: *loc,
                }),
            },
            Expression::Assignment { ident, val } => {
                let val = self.evaluate(val, scope.clone())?;
                scope
                    .clone()
                    .borrow_mut()
                    .assign(ident.identifier(), val)
                    .ok_or(Error::VarNotFound(ident.clone()))
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
