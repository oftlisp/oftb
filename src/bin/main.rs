#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate oftb;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

mod options;

use failure::Error;
use oftb::{parse_file, Heap};
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
    let mut heap = Heap::new();

    let addr = parse_file(options.path, &mut heap)?;
    println!("{}", heap.get(addr));

    Ok(())
}
