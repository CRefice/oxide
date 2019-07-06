use std::rc::Rc;

use crate::vm::{VirtualMachine, Value};

fn print(vals: &[Value]) -> Value {
    println!("{}", vals[0]);
    Value::Null
}

pub fn load_libraries(vm: &mut VirtualMachine) {
    vm.define("print".to_owned(), Value::NativeFn {
        f: Rc::new(print),
        arity: 1,
    });
}
