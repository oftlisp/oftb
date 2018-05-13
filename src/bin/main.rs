#[macro_use]
extern crate failure;
#[macro_use]
extern crate human_panic;
#[macro_use]
extern crate log;
extern crate oftb;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

mod compile;
mod interpret;
mod options;
mod run;

use std::process::exit;

use structopt::StructOpt;

use options::{Options, Subcommand};

fn main() {
    let options = Options::from_args();
    options.start_logger();
    if !options.quiet && options.verbose == 0 {
        setup_panic!();
    }

    let result = match options.subcommand {
        Subcommand::Compile(options) => compile::run(options),
        Subcommand::Interpret(options) => interpret::run(options),
        Subcommand::Run(options) => run::run(options),
    };

    if let Err(err) = result {
        let mut first = true;
        let num_errs: usize = err.causes().map(|_| 1).sum();
        if num_errs <= 1 {
            error!("{}", err);
        } else {
            for cause in err.causes() {
                if first {
                    first = false;
                    error!("           {}", cause);
                } else {
                    error!("caused by: {}", cause);
                }
            }
        }
        debug!("{}", err.backtrace());
        exit(1);
    }
}
