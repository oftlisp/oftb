extern crate failure;
#[macro_use]
extern crate human_panic;
#[macro_use]
extern crate log;
extern crate oftb;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

mod options;

use std::fs::File;
use std::process::exit;

use failure::Error;
use oftb::intrinsics::Intrinsics;
use oftb::modules::Packages;
use structopt::StructOpt;

use options::Options;

fn main() {
    let options = Options::from_args();
    options.start_logger();
    if !options.quiet && options.verbose == 0 {
        setup_panic!();
    }

    if let Err(err) = run(options) {
        let mut first = true;
        for cause in err.causes() {
            if first {
                first = false;
                error!("           {}", cause);
            } else {
                error!("caused by: {}", cause);
            }
        }
        debug!("{}", err.backtrace());
        exit(1);
    }
}

fn run(options: Options) -> Result<(), Error> {
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
