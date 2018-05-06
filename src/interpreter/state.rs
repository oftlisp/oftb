use interpreter::{Store, Value};
use interpreter::control::Control;
use interpreter::env::Env;
use interpreter::kont::Kont;

/// The state of the interpreter.
#[derive(Debug)]
pub enum State<'program> {
    /// A running CESK machine.
    Running(
        Control<'program>,
        Env<'program>,
        Store<'program>,
        Vec<Kont<'program>>,
    ),

    /// A halted state, where the evaluation has proceeded to the value.
    Halted(Value<'program>),
}
