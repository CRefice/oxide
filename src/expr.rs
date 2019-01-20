use crate::token::Token;
use crate::value::Value;

#[derive(Clone)]
pub enum Expression {
    Literal(Value),
    Variable(Token),
    Grouping(Box<Expression>),
    Array(Vec<Expression>),
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
        loc: (usize, usize),
    },
    Indexing {
        operand: Box<Expression>,
        index: Box<Expression>,
        loc: (usize, usize)
    },
    Assignment {
        ident: Token,
        val: Box<Expression>,
    },
    IndexingAssignment {
        ident: Token,
        index: Box<Expression>,
        val: Box<Expression>,
        loc: (usize, usize)
    },
    Unary {
        op: Token,
        operand: Box<Expression>,
    },
    Binary {
        a: Box<Expression>,
        b: Box<Expression>,
        op: Token,
    },
    Logical {
        a: Box<Expression>,
        b: Box<Expression>,
        op: Token,
    },
}
