use crate::token::Token;
use crate::value::Value;

#[derive(Clone)]
pub enum Expression<'a> {
    Literal(Value<'a>),
    Variable(Token<'a>),
    Grouping(Box<Expression<'a>>),
    Call {
        callee: Box<Expression<'a>>,
        args: Vec<Expression<'a>>,
        loc: (usize, usize),
    },
    Assignment {
        ident: Token<'a>,
        val: Box<Expression<'a>>,
    },
    Unary {
        op: Token<'a>,
        operand: Box<Expression<'a>>,
    },
    Binary {
        a: Box<Expression<'a>>,
        b: Box<Expression<'a>>,
        op: Token<'a>,
    },
    Logical {
        a: Box<Expression<'a>>,
        b: Box<Expression<'a>>,
        op: Token<'a>,
    },
}
