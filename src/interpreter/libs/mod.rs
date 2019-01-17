pub mod io;
pub mod conv;

use crate::environment::Scope;

pub fn load_libs(s: &mut Scope) {
    io::load_libs(s);
    conv::load_libs(s);
}
