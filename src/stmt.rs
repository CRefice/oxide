use crate::expr;

#[derive(Debug)]
pub enum Statement <'a> {
    VarDecl(&'a str, expr::Expression <'a>),
    Expression(expr::Expression<'a>),
}
