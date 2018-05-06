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

use failure::Error;
use oftb::flatanf::Program;
use structopt::StructOpt;

use options::Options;

fn main() {
    let options = Options::from_args();
    options.start_logger();
    if !options.quiet && options.verbose == 0 {
        setup_panic!();
    }

    if let Err(err) = run(options) {
        error!("{}", err)
    }
}

fn run(options: Options) -> Result<(), Error> {
    // Load the bytecode file.
    let program = {
        let mut f = File::open(options.file)?;
        Program::deserialize_from(&mut f)
    };

    let mut interpreter = Interpreter::new(&program);
    unimplemented!("{:#?}", bc)
}
