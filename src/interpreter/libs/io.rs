use std::io;

use crate::environment::Scope;

pub fn load_libs(s: &mut Scope) {
    s.define("read_line", function!(, {
        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();
        Ok(Value::Str(s.trim().to_owned()))
    }));
    s.define("print", function!(a, {
        print!("{}", a);
        Ok(Value::Void)
    }));
    s.define("println", function!(a, {
        println!("{}", a);
        Ok(Value::Void)
    }));
}
