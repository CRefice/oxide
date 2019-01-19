use crate::environment::Scope;
use crate::value;

pub fn load_libs<'a>(s: &mut Scope<'a>) {
    s.define(
        "num",
        function!(a, {
            match a {
                x @ Value::Num(_) => Ok(x),
                Value::Bool(b) => Ok(Value::Num(if b { 1.0 } else { 0.0 })),
                Value::Str(s) => Ok(Value::Num(s.parse().unwrap())),
                x => Err(value::Error::WrongType(Value::Num(0.0), x)),
            }
        }),
    );
}
