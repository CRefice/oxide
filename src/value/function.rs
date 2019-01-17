use std::fmt::{self, Debug, Formatter};
use std::ops;

use super::Value;
use crate::environment::ScopeHandle;
use crate::stmt::Statement;
use crate::token::Token;

#[derive(Clone)]
pub enum Fn<'a> {
    Native {
        arity: usize,
        f: &'a dyn ops::Fn(Vec<Value<'a>>) -> Value<'a>,
    },
    User {
        closure: ScopeHandle<'a>,
        params: Vec<Token<'a>>,
        body: Box<Statement<'a>>,
    },
}

macro_rules! replace_expr {
    ($_t:ident $sub:expr) => {
        $sub
    };
}

macro_rules! count_ids {
        ($($tts:ident)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
}

#[macro_export]
macro_rules! function {
    ( $($x:ident),* , $body:expr ) => {
        {
        use crate::value::{Value, Fn};
            Value::Fn(Fn::Native{
                f: &|vec: Vec<Value>| {
                let mut _i = vec.into_iter();
                $(
                    let $x = _i.next().unwrap();
                )*
                $body
            },
            arity: count_ids!($($x)*)
            })
        }
    };
}

impl<'a> Debug for Fn<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Fn::Native { .. } => write!(f, "<NativeFn>"),
            Fn::User { .. } => write!(f, "<UserFn>"),
        }
    }
}
