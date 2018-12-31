use crate::expr;
use crate::value::Value;

enum Statement <'a> {
    Expression(expr::Expression),
    VarDecl(&'a str, Value)
}
