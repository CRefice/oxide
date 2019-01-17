use std::io;

use crate::environment::Scope;

pub fn load_libs<'a>(s: &mut Scope<'a>) {
    s.define("read_line", function!(, {
        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();
        Value::Str(s.trim().to_owned())
    }));
    s.define("print", function!(a, {
        print!("{}", a);
        Value::Void
    }));
    s.define("println", function!(a, {
        println!("{}", a);
        Value::Void
    }));
}
