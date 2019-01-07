use crate::expr;
use crate::scan::Token;

#[derive(Clone)]
pub enum Statement<'a> {
    VarDecl {
        ident: Token<'a>,
        init: expr::Expression<'a>,
    },
    FnDecl {
        ident: Token<'a>,
        params: Vec<Token<'a>>,
        body: Box<Statement<'a>>
    },
    If {
        cond: expr::Expression<'a>,
        succ: Box<Statement<'a>>,
        fail: Option<Box<Statement<'a>>>,
    },
    While {
        cond: expr::Expression<'a>,
        stmt: Box<Statement<'a>>,
    },
    Expression(expr::Expression<'a>),
    Block(Vec<Statement<'a>>),
}
