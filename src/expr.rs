use crate::token::Token;
use crate::value::Value;

#[derive(Clone)]
pub enum Expression<'a> {
    Literal(Value<'a>),
    Variable(Token<'a>),
    Call {
        callee: Box<Expression<'a>>,
        args: Vec<Expression<'a>>,
    },
    Assignment {
        ident: Token<'a>,
        val: Box<Expression<'a>>,
    },
    Grouping(Box<Expression<'a>>),
    Unary(Token<'a>, Box<Expression<'a>>),
    Binary(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
    Logical(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
}
