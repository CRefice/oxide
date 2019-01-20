use crate::environment::Scope;
use crate::value;

pub fn load_libs(s: &mut Scope) {
    s.define(
        "len",
        function!(a, {
            match a {
                Value::Array(x) => Ok(Value::Num(x.len() as f64)),
                Value::Str(s) => Ok(Value::Num(s.len() as f64)),
                x => Err(value::Error::WrongType(Value::Array(Vec::new()), x)),
            }
        }),
    );
}
