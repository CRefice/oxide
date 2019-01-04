use crate::expr;

#[derive(Debug)]
pub enum Statement<'a> {
    VarDecl {
        name: &'a str,
        init: expr::Expression<'a>,
    },
    If {
        cond: expr::Expression<'a>,
        succ: Box<Statement<'a>>,
        fail: Option<Box<Statement<'a>>>,
    },
    Expression(expr::Expression<'a>),
    Block(Vec<Statement<'a>>),
}
