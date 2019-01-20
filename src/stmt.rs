use crate::expr;
use crate::token::Token;

#[derive(Clone)]
pub enum Statement {
    VarDecl {
        ident: Token,
        init: expr::Expression,
    },
    FnDecl {
        ident: Token,
        params: Vec<Token>,
        body: Box<Statement>,
    },
    If {
        loc: (usize, usize),
        cond: expr::Expression,
        succ: Box<Statement>,
        fail: Option<Box<Statement>>,
    },
    While {
        loc: (usize, usize),
        cond: expr::Expression,
        stmt: Box<Statement>,
    },
    Return(Option<expr::Expression>),
    Expression(expr::Expression),
    Block(Vec<Statement>),
}
