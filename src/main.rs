mod compile;
mod interp;
mod loc;
mod scan;
mod vm;

use std::env::args;

fn main() {
    if let Some(path) = args().nth(1) {
        if let Err(e) = interp::run_file(path) {
            println!("{}", e);
        }
    } else {
        interp::repl();
    }
}
