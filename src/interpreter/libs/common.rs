use crate::environment::Scope;

pub fn load_libs<'a>(s: &mut Scope<'a>) {
    s.define(
        "len",
        function!(a, {
            match a {
                Value::Array(x) => Value::Num(x.len() as f64),
                Value::Str(s) => Value::Num(s.len() as f64),
                x => panic!("Can't get length out of {}", x),
            }
        }),
    );
}
