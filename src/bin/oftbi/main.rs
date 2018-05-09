extern crate failure;
#[macro_use]
extern crate human_panic;
#[macro_use]
extern crate log;
extern crate oftb;
extern crate stderrlog;
#[macro_use]
extern crate structopt;
extern crate symbol;

mod options;

use std::fs::File;
use std::process::exit;

use failure::Error;
use oftb::Literal;
use oftb::flatanf::{AExpr, CExpr, Expr, Program};
use oftb::interpreter::Interpreter;
use oftb::intrinsics::Intrinsics;
use structopt::StructOpt;

use options::Options;

fn main() {
    let options = Options::from_args();
    options.start_logger();
    if !options.quiet && options.verbose == 0 {
        setup_panic!();
    }

    if let Err(err) = run(options) {
        error!("{}", err);
        exit(1);
    }
}

fn run(options: Options) -> Result<(), Error> {
    // Load the bytecode file.
    let program = {
        let mut f = File::open(options.file)?;
        Program::deserialize_from(&mut f)?
    };

    // Create the expression for the call to main.
    let args = options.args.into_iter().map(Literal::String).collect();
    let args = AExpr::Literal(Literal::list(args));
    let main =
        Expr::CExpr(CExpr::Call(AExpr::Global("main:main".into()), vec![args]));

    // Create the interpreter.
    let mut interpreter = Interpreter::new();
    interpreter.add_builtins::<Intrinsics>();

    // Start interpreting global decls.
    debug!("Initializing program...");
    for &(name, ref expr) in &program.decls {
        debug!("{} = {:?}", name, expr);
        let val = interpreter.eval(expr);
        interpreter.globals.insert(name, val);
    }

    // Call main.
    debug!("Running program...");
    let retval = interpreter.eval(&main);
    println!("{:#?}", retval);

    Ok(())
}
