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
    let meta = pkgs.load_metadata_from(&options.path)?;
    debug!("Compiling {:#?}", meta);
    pkgs.add_modules_from(options.path)?;

    let program = pkgs.compile(meta.name.into(), &options.binary)?;
    info!("{:#?}", program);

    Ok(())
}
