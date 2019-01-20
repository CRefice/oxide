use std::fmt::{self, Debug, Formatter};
use std::ops;
use std::rc::Rc;

use super::{Result, Value};
use crate::environment::ScopeHandle;
use crate::stmt::Statement;
use crate::token::Token;

#[derive(Clone)]
pub enum Fn {
    Native {
        arity: usize,
        f: Rc<dyn ops::Fn(Vec<Value>) -> Result<Value>>,
    },
    User {
        closure: ScopeHandle,
        params: Vec<Token>,
        body: Box<Statement>,
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
                f: std::rc::Rc::new(|vec: Vec<Value>| {
                let mut _i = vec.into_iter();
                $(
                    let $x = _i.next().unwrap();
                )*
                $body
            }),
            arity: count_ids!($($x)*)
            })
        }
    };
}

impl Debug for Fn {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Fn::Native { .. } => write!(f, "<NativeFn>"),
            Fn::User { .. } => write!(f, "<UserFn>"),
        }
    }
}
