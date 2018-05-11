use std::collections::HashSet;

use flatanf::Program;
use {Error, ErrorKind};

/// Checks that the intrinsics the outputted program declares are a subset of
/// those we support.
pub fn globals_exist(program: &Program) -> Result<(), Error> {
    let mut declared = program.intrinsics.clone();
    let mut referenced = HashSet::new();
    for &(name, ref expr) in &program.decls {
        declared.insert(name);
        referenced.extend(expr.global_vars());
    }

    referenced.retain(|x| !declared.contains(x));
    if referenced.is_empty() {
        Ok(())
    } else {
        let mut free = Vec::with_capacity(referenced.len());
        free.extend(referenced);
        free.sort();
        Err(ErrorKind::FreeVars(free).into())
    }
}
