use symbol::Symbol;

use flatanf::Program;
use {Error, ErrorKind};

/// Checks that the outputted program declares a `main:main` function.
pub fn main_exists(program: &Program) -> Result<(), Error> {
    let main = Symbol::from("main:main");
    if program
        .decls
        .iter()
        .any(|&(name, _)| name == main)
    {
        Ok(())
    } else {
        Err(ErrorKind::NoMainFunction.into())
    }
}
