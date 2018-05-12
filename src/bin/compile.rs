use std::fs::File;

use failure::Error;
use oftb::intrinsics::Intrinsics;
use oftb::modules::Packages;

use options::CompileOptions;

pub fn run(options: CompileOptions) -> Result<(), Error> {
    let mut pkgs = Packages::new();
    pkgs.add_builtins::<Intrinsics>();

    // Load the stdlib.
    info!("Loading stdlib...");
    pkgs.add_stdlib_from(options.std_path())?;

    // Load the package being compiled.
    info!("Loading main package...");
    let name = pkgs.add_modules_from(options.package_path.clone())?;

    // Compile the package.
    info!("Compiling {}...", name);
    let program = pkgs.compile(name, &options.binary_name)?;

    // Write out the compiled program.
    info!("Saving bytecode...");
    let path = options.output_path()?;
    let mut f = File::create(path)?;
    program.serialize_to(&mut f)?;

    // Done!
    Ok(())
}
