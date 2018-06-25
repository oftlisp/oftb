use std::fs::File;

use failure::Error;
use oftb::flatanf::{AExpr, CExpr, Expr, Program};
use oftb::interpreter::{Interpreter, Value};
use oftb::intrinsics::Intrinsics;
use oftb::Literal;

use options::InterpretOptions;

pub fn run(options: InterpretOptions) -> Result<(), Error> {
    // Load the bytecode file.
    let program = {
        let mut f = File::open(options.file)?;
        Program::deserialize_from(&mut f)?
    };
    trace!("{:#?}", program);
    if !program
        .decls
        .iter()
        .map(|(name, _)| name)
        .any(|name| name.as_str() == "main:main")
    {
        bail!("Missing main:main function.")
    }

    // Create the expression for the call to main.
    let args = options.args.into_iter().map(Literal::String).collect();
    let args = AExpr::Literal(Literal::list(args));
    let main = Expr::CExpr(CExpr::Call(AExpr::Global("main:main".into()), vec![args]));

    // Create the interpreter.
    let mut interpreter = Interpreter::new();
    interpreter.add_builtins::<Intrinsics>();

    // Start interpreting global decls.
    debug!("Initializing program...");
    for &(name, ref expr) in &program.decls {
        let val = interpreter.eval(expr);
        if let Value::Closure(addr) = val {
            interpreter.store.mutate_closure_name(addr, name);
        }
        interpreter.globals.insert(name, val);
    }

    // Call main.
    debug!("Running program...");
    let retval = interpreter.eval(&main);
    if retval != Value::Nil {
        println!("{}", retval.display(&interpreter.store, false));
    }

    Ok(())
}
