//! Some sanity checks run immediately before claiming a `Program` to be
//! successfully compiled.

mod globals_exist;
mod locals_valid;

use Error;
use flatanf::Program;
use sanity::globals_exist::globals_exist;
use sanity::locals_valid::locals_valid;

/// Runs all sanity checks on a program.
pub fn check(program: &Program) -> Result<(), Error> {
    globals_exist(program)?;
    locals_valid(program)?;
    Ok(())
}
