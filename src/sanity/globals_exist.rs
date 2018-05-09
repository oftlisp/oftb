use Error;
use flatanf::Program;

/// Checks that the intrinsics the outputted program declares are a subset of
/// those we support.
pub fn globals_exist(program: &Program) -> Result<(), Error> {
    warn!("TODO sanity::globals_exist");
    Ok(())
}
