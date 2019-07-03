mod compile;
mod interp;
mod scan;
mod vm;

use std::env::args;
use std::fs::File;

fn main() -> Result<(), interp::Error> {
    let mut interp = interp::Interpreter::new();
    if let Some(path) = args().nth(1) {
        let file = File::open(path)?;
        interp.run_file(file)?;
    } else {
        interp.repl()?;
    }
    Ok(())
}
