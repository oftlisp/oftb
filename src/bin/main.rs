extern crate failure;
#[macro_use]
extern crate log;
extern crate oftb;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

mod options;

use failure::Error;
use oftb::modules::Packages;
use structopt::StructOpt;

use options::Options;

fn main() {
    let options = Options::from_args();
    options.start_logger();

    if let Err(err) = run(options) {
        error!("{}", err)
    }
}

fn run(options: Options) -> Result<(), Error> {
    let mut pkgs = Packages::new();

    // Load the stdlib.
    info!("Loading stdlib...");
    pkgs.add_stdlib_from(options.std_path())?;

    // Load the package being compiled.
    info!("Loading main package...");
    let name = pkgs.add_modules_from(options.path)?;

    // Compile the package.
    info!("Compiling {}...", name);
    let program = pkgs.compile(name, &options.binary)?;
    info!("{:#?}", program);

    Ok(())
}
