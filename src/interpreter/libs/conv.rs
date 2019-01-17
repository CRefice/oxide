use crate::environment::Scope;

pub fn load_libs<'a>(s: &mut Scope<'a>) {
    s.define(
        "num",
        function!(a, {
            match a {
                Value::Void => Value::Num(0.0),
                x @ Value::Num(_) => x,
                Value::Bool(b) => Value::Num(if b { 1.0 } else { 0.0 }),
                Value::Str(s) => Value::Num(s.parse().unwrap()),
                Value::Fn(_) => panic!("Tried to get num out of function"),
            }
        }),
    );
}
