use failure::Error;
use oftb::Literal;
use oftb::flatanf::{AExpr, CExpr, Expr};
use oftb::interpreter::{Interpreter, Value};
use oftb::intrinsics::Intrinsics;
use oftb::modules::Packages;

use options::RunOptions;

pub fn run(options: RunOptions) -> Result<(), Error> {
    let mut pkgs = Packages::new();
    pkgs.add_builtins::<Intrinsics>();

    // Load the stdlib.
    debug!("Loading stdlib...");
    pkgs.add_stdlib_from(options.std_path())?;

    // Load the package being compiled.
    debug!("Loading main package...");
    let name = pkgs.add_modules_from(options.package_path.clone())?;

    // Compile the package.
    debug!("Compiling {}...", name);
    let program = pkgs.compile(name, &options.binary_name)?;

    // Create the expression for the call to main.
    let args = options
        .args
        .into_iter()
        .map(Literal::String)
        .collect();
    let args = AExpr::Literal(Literal::list(args));
    let main = Expr::CExpr(CExpr::Call(
        AExpr::Global("main:main".into()),
        vec![args],
    ));

    // Create the interpreter.
    let mut interpreter = Interpreter::new();
    interpreter.add_builtins::<Intrinsics>();

    // Start interpreting global decls.
    debug!("Initializing program...");
    for &(name, ref expr) in &program.decls {
        let val = interpreter.eval(expr);
        interpreter.globals.insert(name, val);
    }

    // Call main.
    debug!("Running program...");
    let retval = interpreter.eval(&main);
    if retval != Value::Nil {
        println!("{}", retval.display(&interpreter.store, false));
    }

    // Done!
    Ok(())
}
