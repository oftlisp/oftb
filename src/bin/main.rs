extern crate failure;
#[macro_use]
extern crate log;
extern crate oftb;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

mod options;

use failure::Error;
use oftb::parse_file;
use oftb::anf::Module as AnfModule;
use oftb::ast::Module as AstModule;
use oftb::flatanf::Program;
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
    let literals = parse_file(options.path)?;
    info!("Literals:");
    for l in &literals {
        info!("{}", l);
    }

    let ast = AstModule::from_values(literals)?;
    println!();
    info!("AST Module:");
    info!("{:?}", ast);

    let anf = AnfModule::from(ast);
    println!();
    info!("ANF Module:");
    info!("{:?}", anf);

    // TODO: Resolve dependencies.
    let modules = vec![anf];

    let program = Program::from_modules(modules, Default::default());
    println!();
    info!("FlatANF Program:");
    info!("{:?}", program);

    Ok(())
}
