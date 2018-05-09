use interpreter::{Control, Kont, Value};
use interpreter::env::Env;

/// The state of the interpreter.
#[derive(Debug)]
pub enum State<'program> {
    /// A running CESK machine.
    Running(Control<'program>, Env, Vec<Kont<'program>>),

    /// A halted state, where the evaluation has proceeded to the value.
    Halted(Value),
}
