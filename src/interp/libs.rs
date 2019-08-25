use std::rc::Rc;

use crate::vm::{Value, ValueError, VirtualMachine};

fn print(vals: &[Value]) -> Result<Value, ValueError> {
    println!("{}", vals[0]);
    Ok(Value::Null)
}

pub fn load_libraries(vm: &mut VirtualMachine) {
    vm.define(
        "print".to_owned(),
        Value::NativeFn {
            f: Rc::new(print),
            arity: 1,
        },
    );
}
