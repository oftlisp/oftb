use flatanf::Program;

/// Checks that all referenced local variables would actually exist.
///
/// This check panics if it fails -- a failed test here indicates a compiler
/// error.
pub fn locals_valid(program: &Program) {
    warn!("TODO sanity::locals_valid");
}
