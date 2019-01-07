use crate::scan::Token;
use crate::value::Value;

#[derive(Debug)]
pub enum Expression<'a> {
    Literal(Value),
    Variable(Token<'a>),
    Assignment{ident: Token<'a>, val: Box<Expression<'a>>},
    Grouping(Box<Expression<'a>>),
    Unary(Token<'a>, Box<Expression<'a>>),
    Binary(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
    Logical(Box<Expression<'a>>, Token<'a>, Box<Expression<'a>>),
}
