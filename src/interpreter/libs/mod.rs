pub mod common;
pub mod conv;
pub mod io;

use crate::environment::Scope;

pub fn load_libs(s: &mut Scope) {
    common::load_libs(s);
    conv::load_libs(s);
    io::load_libs(s);
}
